/**
 * rusty-merge.js - Automerge-based Real-time Sync Client
 *
 * Features:
 * - WebSocket real-time sync with Automerge CRDT
 * - Automatic conflict-free merging
 * - Offline-first with IndexedDB persistence
 * - Automatic reconnection with exponential backoff
 * - Multi-tab sync via BroadcastChannel
 * - Full CRUD operations
 * - Optimistic UI updates
 *
 * Usage:
 * <script src="/api/merge/client.js"
 *         data-entities="users,posts"
 *         data-debug="false">
 * </script>
 *
 * // Or programmatically:
 * const client = new MergeClient({ entities: ['users', 'posts'] });
 * await client.connect();
 * await client.create('users', { name: 'Alice' });
 */

(function() {
    'use strict';

    const ConnectionState = {
        DISCONNECTED: 'disconnected',
        CONNECTING: 'connecting',
        CONNECTED: 'connected',
        RECONNECTING: 'reconnecting'
    };

    class MergeClient {
        constructor(config = {}) {
            this.entities = config.entities || [];
            this.wsUrl = config.wsUrl || this._defaultWsUrl();
            this.debug = config.debug || false;

            // Connection state
            this.connectionState = ConnectionState.DISCONNECTED;
            this.ws = null;
            this.reconnectAttempts = 0;
            this.maxReconnectAttempts = 10;
            this.reconnectDelay = 1000;
            this.maxReconnectDelay = 30000;

            // Request tracking
            this.pendingRequests = new Map();
            this.requestIdCounter = 0;

            // Sync state
            this.syncState = new Map(); // entity -> { heads: [], data: Map }
            this.db = null;

            // Multi-tab
            this.broadcastChannel = null;
            this.tabId = this._generateTabId();

            // Event handlers
            this.eventHandlers = new Map();

            this._log('MergeClient initialized', { entities: this.entities });
        }

        // =====================================================================
        // Logging
        // =====================================================================

        _log(...args) {
            if (this.debug) {
                console.log('[MergeClient]', ...args);
            }
        }

        _error(...args) {
            console.error('[MergeClient]', ...args);
        }

        // =====================================================================
        // Initialization
        // =====================================================================

        _defaultWsUrl() {
            const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
            return `${protocol}//${location.host}/api/merge/ws`;
        }

        _generateTabId() {
            return `tab_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        }

        async init() {
            await this._initIndexedDB();
            await this._loadSyncState();
            this._setupBroadcastChannel();
            this._setupOfflineHandlers();
        }

        async _initIndexedDB() {
            return new Promise((resolve, reject) => {
                const request = indexedDB.open('rusty-merge', 1);

                request.onerror = () => {
                    this._error('Failed to open IndexedDB', request.error);
                    reject(request.error);
                };

                request.onsuccess = () => {
                    this.db = request.result;
                    this._log('IndexedDB initialized');
                    resolve();
                };

                request.onupgradeneeded = (event) => {
                    const db = event.target.result;

                    // Store for each entity type
                    for (const entity of this.entities) {
                        if (!db.objectStoreNames.contains(entity)) {
                            db.createObjectStore(entity, { keyPath: 'id' });
                        }
                    }

                    // Sync state store
                    if (!db.objectStoreNames.contains('_sync_state')) {
                        db.createObjectStore('_sync_state', { keyPath: 'entity' });
                    }

                    // Pending mutations store
                    if (!db.objectStoreNames.contains('_pending')) {
                        const store = db.createObjectStore('_pending', { autoIncrement: true });
                        store.createIndex('timestamp', 'timestamp');
                    }

                    this._log('IndexedDB schema created');
                };
            });
        }

        async _loadSyncState() {
            for (const entity of this.entities) {
                const state = await this._dbGet('_sync_state', entity);
                if (state) {
                    this.syncState.set(entity, state);
                } else {
                    this.syncState.set(entity, { entity, heads: [], count: 0 });
                }
            }
        }

        // =====================================================================
        // Connection Management
        // =====================================================================

        async connect() {
            if (this.connectionState === ConnectionState.CONNECTING ||
                this.connectionState === ConnectionState.CONNECTED) {
                return;
            }

            await this.init();
            this._connectWebSocket();
        }

        _connectWebSocket() {
            this._updateConnectionState(ConnectionState.CONNECTING);
            this._log('Connecting to WebSocket:', this.wsUrl);

            try {
                this.ws = new WebSocket(this.wsUrl);

                this.ws.onopen = () => {
                    this._log('WebSocket connected');
                    this._updateConnectionState(ConnectionState.CONNECTED);
                    this.reconnectAttempts = 0;
                    this.reconnectDelay = 1000;

                    // Subscribe to entities with current sync state
                    const syncStateMap = {};
                    for (const [entity, state] of this.syncState) {
                        syncStateMap[entity] = state.heads || [];
                    }

                    this._send({
                        type: 'subscribe',
                        entities: this.entities,
                        sync_state: syncStateMap
                    });

                    // Process pending mutations
                    this._syncPendingMutations();
                };

                this.ws.onmessage = async (event) => {
                    try {
                        const msg = JSON.parse(event.data);
                        await this._handleMessage(msg);
                    } catch (error) {
                        this._error('Failed to handle message:', error);
                    }
                };

                this.ws.onerror = (error) => {
                    this._error('WebSocket error:', error);
                };

                this.ws.onclose = () => {
                    this._log('WebSocket closed');
                    this._handleDisconnect();
                };
            } catch (error) {
                this._error('Failed to create WebSocket:', error);
                this._handleDisconnect();
            }
        }

        _handleDisconnect() {
            this._updateConnectionState(ConnectionState.DISCONNECTED);

            if (this.reconnectAttempts < this.maxReconnectAttempts) {
                this._reconnect();
            } else {
                this._error('Max reconnect attempts reached');
                this._emit('error', { message: 'Connection failed' });
            }
        }

        _reconnect() {
            this.reconnectAttempts++;
            this._updateConnectionState(ConnectionState.RECONNECTING);

            const delay = Math.min(
                this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1),
                this.maxReconnectDelay
            );

            // Add jitter
            const jitter = delay * 0.2 * Math.random();
            const actualDelay = delay + jitter;

            this._log(`Reconnecting in ${Math.round(actualDelay)}ms (attempt ${this.reconnectAttempts})`);

            setTimeout(() => this._connectWebSocket(), actualDelay);
        }

        _updateConnectionState(newState) {
            const oldState = this.connectionState;
            this.connectionState = newState;

            if (oldState !== newState) {
                this._log(`Connection state: ${oldState} -> ${newState}`);
                this._emit('connection:state', { state: newState, oldState });
            }
        }

        disconnect() {
            if (this.ws) {
                this.ws.close();
                this.ws = null;
            }
            this._updateConnectionState(ConnectionState.DISCONNECTED);
        }

        // =====================================================================
        // Message Handling
        // =====================================================================

        _send(message) {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(JSON.stringify(message));
                return true;
            }
            return false;
        }

        _generateRequestId() {
            return `req_${++this.requestIdCounter}_${Date.now()}`;
        }

        async _request(message) {
            const requestId = this._generateRequestId();
            message.request_id = requestId;

            return new Promise((resolve, reject) => {
                const timeout = setTimeout(() => {
                    this.pendingRequests.delete(requestId);
                    reject(new Error('Request timeout'));
                }, 30000);

                this.pendingRequests.set(requestId, { resolve, reject, timeout });

                if (!this._send(message)) {
                    clearTimeout(timeout);
                    this.pendingRequests.delete(requestId);
                    reject(new Error('Not connected'));
                }
            });
        }

        async _handleMessage(msg) {
            this._log('Received:', msg.type);

            switch (msg.type) {
                case 'subscribed':
                    this._log('Subscribed to:', msg.entities);
                    this._emit('subscribed', msg);
                    break;

                case 'sync_response':
                    await this._handleSyncResponse(msg);
                    break;

                case 'change':
                    await this._handleChange(msg.change);
                    break;

                case 'ack':
                    this._handleAck(msg);
                    break;

                case 'pong':
                    // Heartbeat response
                    break;

                case 'error':
                    this._error('Server error:', msg.message);
                    this._emit('error', msg);
                    break;
            }
        }

        async _handleSyncResponse(msg) {
            const { entity, update, heads, count } = msg;

            // Decode base64 update
            const updateBytes = this._base64ToBytes(update);

            // Update sync state
            const state = this.syncState.get(entity) || { entity, heads: [], count: 0 };
            state.heads = heads;
            state.count = count;
            this.syncState.set(entity, state);

            // Save sync state
            await this._dbPut('_sync_state', state);

            this._log(`Synced ${entity}: ${count} entities, heads: ${heads.length}`);
            this._emit('sync', { entity, count, heads });
        }

        async _handleChange(change) {
            const { entity_type, entity_id, change_type, data } = change;

            // Update local cache
            if (change_type === 'delete') {
                await this._dbDelete(entity_type, entity_id);
            } else if (data) {
                await this._dbPut(entity_type, { id: entity_id, ...data });
            }

            // Emit change event
            this._emit('change', change);
            this._emit(`${entity_type}:change`, change);

            // Broadcast to other tabs
            this._broadcastChange(change);
        }

        _handleAck(msg) {
            const pending = this.pendingRequests.get(msg.request_id);
            if (pending) {
                clearTimeout(pending.timeout);
                this.pendingRequests.delete(msg.request_id);

                if (msg.success) {
                    pending.resolve(msg.data);
                } else {
                    pending.reject(new Error(msg.error || 'Request failed'));
                }
            }
        }

        // =====================================================================
        // CRUD Operations
        // =====================================================================

        async create(entity, data) {
            const id = data.id || this._generateId();
            const entityData = { id, ...data };

            // Optimistic update
            await this._dbPut(entity, entityData);
            this._emit(`${entity}:change`, {
                entity_type: entity,
                entity_id: id,
                change_type: 'create',
                data: entityData
            });

            if (this.connectionState === ConnectionState.CONNECTED) {
                try {
                    const result = await this._request({
                        type: 'create',
                        entity,
                        id,
                        data
                    });
                    return result || entityData;
                } catch (error) {
                    // Queue for later
                    await this._queueMutation({ type: 'create', entity, id, data });
                    return entityData;
                }
            } else {
                await this._queueMutation({ type: 'create', entity, id, data });
                return entityData;
            }
        }

        async update(entity, id, data) {
            // Optimistic update
            const existing = await this._dbGet(entity, id);
            const updated = { ...existing, ...data, id };
            await this._dbPut(entity, updated);

            this._emit(`${entity}:change`, {
                entity_type: entity,
                entity_id: id,
                change_type: 'update',
                data: updated
            });

            if (this.connectionState === ConnectionState.CONNECTED) {
                try {
                    return await this._request({
                        type: 'update',
                        entity,
                        id,
                        data
                    });
                } catch (error) {
                    await this._queueMutation({ type: 'update', entity, id, data });
                    return updated;
                }
            } else {
                await this._queueMutation({ type: 'update', entity, id, data });
                return updated;
            }
        }

        async updateField(entity, id, field, value) {
            return this.update(entity, id, { [field]: value });
        }

        async delete(entity, id) {
            // Optimistic delete
            await this._dbDelete(entity, id);

            this._emit(`${entity}:change`, {
                entity_type: entity,
                entity_id: id,
                change_type: 'delete',
                data: null
            });

            if (this.connectionState === ConnectionState.CONNECTED) {
                try {
                    await this._request({
                        type: 'delete',
                        entity,
                        id
                    });
                    return true;
                } catch (error) {
                    await this._queueMutation({ type: 'delete', entity, id });
                    return true;
                }
            } else {
                await this._queueMutation({ type: 'delete', entity, id });
                return true;
            }
        }

        async get(entity, id) {
            return this._dbGet(entity, id);
        }

        async list(entity) {
            return this._dbGetAll(entity);
        }

        async query(entity, filter) {
            const all = await this.list(entity);
            if (!filter) return all;

            return all.filter(item => {
                for (const [key, value] of Object.entries(filter)) {
                    if (item[key] !== value) return false;
                }
                return true;
            });
        }

        // =====================================================================
        // IndexedDB Operations
        // =====================================================================

        _dbGet(store, key) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(store, 'readonly');
                const objStore = tx.objectStore(store);
                const request = objStore.get(key);

                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            });
        }

        _dbGetAll(store) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(store, 'readonly');
                const objStore = tx.objectStore(store);
                const request = objStore.getAll();

                request.onsuccess = () => resolve(request.result || []);
                request.onerror = () => reject(request.error);
            });
        }

        _dbPut(store, value) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(store, 'readwrite');
                const objStore = tx.objectStore(store);
                const request = objStore.put(value);

                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error);
            });
        }

        _dbDelete(store, key) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(store, 'readwrite');
                const objStore = tx.objectStore(store);
                const request = objStore.delete(key);

                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error);
            });
        }

        // =====================================================================
        // Offline Queue
        // =====================================================================

        async _queueMutation(mutation) {
            mutation.timestamp = Date.now();
            mutation.id = this._generateId();

            return new Promise((resolve, reject) => {
                const tx = this.db.transaction('_pending', 'readwrite');
                const store = tx.objectStore('_pending');
                const request = store.add(mutation);

                request.onsuccess = () => {
                    this._log('Queued mutation:', mutation.type, mutation.entity);
                    resolve();
                };
                request.onerror = () => reject(request.error);
            });
        }

        async _syncPendingMutations() {
            const pending = await this._dbGetAll('_pending');

            if (pending.length === 0) return;

            this._log(`Processing ${pending.length} pending mutations`);

            for (const mutation of pending) {
                try {
                    switch (mutation.type) {
                        case 'create':
                            await this._request({
                                type: 'create',
                                entity: mutation.entity,
                                id: mutation.id,
                                data: mutation.data
                            });
                            break;
                        case 'update':
                            await this._request({
                                type: 'update',
                                entity: mutation.entity,
                                id: mutation.id,
                                data: mutation.data
                            });
                            break;
                        case 'delete':
                            await this._request({
                                type: 'delete',
                                entity: mutation.entity,
                                id: mutation.id
                            });
                            break;
                    }
                } catch (error) {
                    this._error('Failed to sync mutation:', error);
                }
            }

            // Clear processed mutations
            await this._clearPending();
        }

        async _clearPending() {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction('_pending', 'readwrite');
                const store = tx.objectStore('_pending');
                const request = store.clear();

                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error);
            });
        }

        // =====================================================================
        // Multi-tab Sync
        // =====================================================================

        _setupBroadcastChannel() {
            if (!('BroadcastChannel' in window)) return;

            this.broadcastChannel = new BroadcastChannel('rusty-merge');

            this.broadcastChannel.onmessage = async (event) => {
                if (event.data.tabId === this.tabId) return;

                const { type, change } = event.data;
                if (type === 'change') {
                    // Apply change from another tab
                    const { entity_type, entity_id, change_type, data } = change;

                    if (change_type === 'delete') {
                        await this._dbDelete(entity_type, entity_id);
                    } else if (data) {
                        await this._dbPut(entity_type, { id: entity_id, ...data });
                    }

                    this._emit(`${entity_type}:change`, change);
                }
            };
        }

        _broadcastChange(change) {
            if (this.broadcastChannel) {
                this.broadcastChannel.postMessage({
                    tabId: this.tabId,
                    type: 'change',
                    change
                });
            }
        }

        // =====================================================================
        // Offline Handlers
        // =====================================================================

        _setupOfflineHandlers() {
            window.addEventListener('online', () => {
                this._log('Back online');
                if (this.connectionState !== ConnectionState.CONNECTED) {
                    this._connectWebSocket();
                }
            });

            window.addEventListener('offline', () => {
                this._log('Went offline');
            });
        }

        // =====================================================================
        // Event System
        // =====================================================================

        on(event, handler) {
            if (!this.eventHandlers.has(event)) {
                this.eventHandlers.set(event, []);
            }
            this.eventHandlers.get(event).push(handler);
            return () => this.off(event, handler);
        }

        off(event, handler) {
            const handlers = this.eventHandlers.get(event);
            if (handlers) {
                const index = handlers.indexOf(handler);
                if (index >= 0) handlers.splice(index, 1);
            }
        }

        _emit(event, data) {
            const handlers = this.eventHandlers.get(event);
            if (handlers) {
                handlers.forEach(h => {
                    try {
                        h(data);
                    } catch (error) {
                        this._error('Event handler error:', error);
                    }
                });
            }
        }

        // =====================================================================
        // Utilities
        // =====================================================================

        _generateId() {
            return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, c => {
                const r = Math.random() * 16 | 0;
                const v = c === 'x' ? r : (r & 0x3 | 0x8);
                return v.toString(16);
            });
        }

        _base64ToBytes(base64) {
            const binaryString = atob(base64);
            const bytes = new Uint8Array(binaryString.length);
            for (let i = 0; i < binaryString.length; i++) {
                bytes[i] = binaryString.charCodeAt(i);
            }
            return bytes;
        }

        _bytesToBase64(bytes) {
            let binary = '';
            for (let i = 0; i < bytes.byteLength; i++) {
                binary += String.fromCharCode(bytes[i]);
            }
            return btoa(binary);
        }

        // =====================================================================
        // Static Initialization
        // =====================================================================

        static async init() {
            const scriptTag = document.currentScript;
            if (!scriptTag) return;

            const entities = scriptTag.getAttribute('data-entities');
            const debug = scriptTag.getAttribute('data-debug') === 'true';

            if (!entities) {
                console.error('[MergeClient] No entities specified');
                return;
            }

            const client = new MergeClient({
                entities: entities.split(',').map(e => e.trim()),
                debug
            });

            await client.connect();

            window.mergeClient = client;
            window.dispatchEvent(new CustomEvent('merge:ready', { detail: client }));

            console.log('[MergeClient] Ready');
        }
    }

    // Auto-initialize if data attributes present
    if (document.currentScript?.hasAttribute('data-entities')) {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => MergeClient.init());
        } else {
            MergeClient.init();
        }
    }

    // Export
    window.MergeClient = MergeClient;
})();
