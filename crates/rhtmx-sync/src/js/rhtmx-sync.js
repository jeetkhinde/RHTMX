/**
 * rhtmx-sync.js
 * Client-side IndexedDB synchronization for RHTMX
 *
 * Usage:
 * <script src="/api/sync/client.js"
 *         data-sync-entities="users,posts"
 *         data-conflict-strategy="last-write-wins"
 *         data-debug="false">
 * </script>
 */

(function() {
    'use strict';

    class RHTMXSync {
        constructor(config) {
            this.entities = config.entities || [];
            this.conflictStrategy = config.conflictStrategy || 'last-write-wins';
            this.debug = config.debug || false;
            this.db = null;
            this.eventSource = null;
            this.syncInProgress = false;

            this.log('Initializing RHTMX Sync', { entities: this.entities });
        }

        log(...args) {
            if (this.debug) {
                console.log('[RHTMX Sync]', ...args);
            }
        }

        error(...args) {
            console.error('[RHTMX Sync]', ...args);
        }

        /**
         * Initialize IndexedDB
         */
        async initIndexedDB() {
            return new Promise((resolve, reject) => {
                const request = indexedDB.open('rhtmx-cache', 1);

                request.onerror = () => {
                    this.error('Failed to open IndexedDB', request.error);
                    reject(request.error);
                };

                request.onsuccess = () => {
                    this.db = request.result;
                    this.log('IndexedDB initialized');
                    resolve();
                };

                request.onupgradeneeded = (event) => {
                    const db = event.target.result;

                    // Create object stores for each entity
                    for (const entity of this.entities) {
                        if (!db.objectStoreNames.contains(entity)) {
                            db.createObjectStore(entity, { keyPath: 'id' });
                            this.log(`Created object store: ${entity}`);
                        }
                    }

                    // Create metadata store
                    if (!db.objectStoreNames.contains('_meta')) {
                        db.createObjectStore('_meta', { keyPath: 'key' });
                    }

                    // Create pending mutations store
                    if (!db.objectStoreNames.contains('_pending')) {
                        db.createObjectStore('_pending', { autoIncrement: true });
                    }

                    this.log('IndexedDB schema created');
                };
            });
        }

        /**
         * Perform initial sync for all entities
         */
        async initialSync() {
            this.log('Starting initial sync');

            for (const entity of this.entities) {
                await this.syncEntity(entity);
            }

            this.log('Initial sync complete');
        }

        /**
         * Sync a single entity
         */
        async syncEntity(entity) {
            try {
                // Get last known version
                const lastVersion = await this.getLastVersion(entity);
                this.log(`Syncing ${entity} since version ${lastVersion}`);

                // Fetch changes from server
                const response = await fetch(`/api/sync/${entity}?since=${lastVersion}`);
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();
                this.log(`Received ${data.changes.length} changes for ${entity}`);

                // Apply changes to IndexedDB
                await this.applyChanges(entity, data.changes);

                // Update version
                await this.setLastVersion(entity, data.version);

                this.log(`Synced ${entity} to version ${data.version}`);
            } catch (error) {
                this.error(`Failed to sync ${entity}:`, error);
            }
        }

        /**
         * Apply changes to IndexedDB
         */
        async applyChanges(entity, changes) {
            const tx = this.db.transaction(entity, 'readwrite');
            const store = tx.objectStore(entity);

            for (const change of changes) {
                try {
                    if (change.action === 'delete') {
                        await store.delete(change.entity_id);
                    } else if (change.data) {
                        await store.put(change.data);
                    }
                } catch (error) {
                    this.error(`Failed to apply change to ${entity}:`, error);
                }
            }

            return new Promise((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Get last synced version for entity
         */
        async getLastVersion(entity) {
            const tx = this.db.transaction('_meta', 'readonly');
            const store = tx.objectStore('_meta');
            const request = store.get(`${entity}_version`);

            return new Promise((resolve) => {
                request.onsuccess = () => {
                    const result = request.result;
                    resolve(result ? result.value : 0);
                };
                request.onerror = () => {
                    this.error('Failed to get version', request.error);
                    resolve(0);
                };
            });
        }

        /**
         * Set last synced version for entity
         */
        async setLastVersion(entity, version) {
            const tx = this.db.transaction('_meta', 'readwrite');
            const store = tx.objectStore('_meta');
            await store.put({ key: `${entity}_version`, value: version });

            return new Promise((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Connect to SSE for real-time updates
         */
        connectSSE() {
            this.log('Connecting to SSE');

            this.eventSource = new EventSource('/api/sync/events');

            this.eventSource.addEventListener('sync', async (event) => {
                try {
                    const change = JSON.parse(event.data);
                    this.log('Received SSE update', change);

                    // Apply change to IndexedDB
                    await this.applyChanges(change.entity, [change]);

                    // Update version
                    await this.setLastVersion(change.entity, change.version);

                    // Trigger HTMX refresh
                    this.triggerRefresh(change.entity, change.entity_id);
                } catch (error) {
                    this.error('Failed to process SSE event:', error);
                }
            });

            this.eventSource.onerror = (error) => {
                this.error('SSE connection error:', error);
                // Will auto-reconnect
            };

            this.log('SSE connected');
        }

        /**
         * Trigger HTMX refresh for affected elements
         */
        triggerRefresh(entity, entityId) {
            // Trigger custom event for entity change
            const event = new CustomEvent(`rhtmx:${entity}:changed`, {
                detail: { id: entityId },
                bubbles: true,
            });
            document.body.dispatchEvent(event);
        }

        /**
         * Setup offline mutation handlers
         */
        setupOfflineHandlers() {
            this.log('Setting up offline handlers');

            // Intercept HTMX requests when offline
            document.body.addEventListener('htmx:beforeRequest', async (evt) => {
                if (!navigator.onLine) {
                    this.log('Offline request intercepted', evt.detail);
                    evt.preventDefault();
                    await this.handleOfflineRequest(evt);
                }
            });

            // Sync pending mutations when back online
            window.addEventListener('online', async () => {
                this.log('Back online, syncing pending mutations');
                await this.syncPendingMutations();
            });

            window.addEventListener('offline', () => {
                this.log('Now offline');
            });
        }

        /**
         * Handle request when offline
         */
        async handleOfflineRequest(evt) {
            const target = evt.detail.target;
            const verb = evt.detail.verb;
            const path = evt.detail.path;

            // Extract entity from path (e.g., /api/users -> users)
            const entityMatch = path.match(/\/api\/(\w+)/);
            if (!entityMatch) {
                this.error('Cannot determine entity from path:', path);
                return;
            }

            const entity = entityMatch[1];

            // Get form data if POST/PUT/PATCH
            if (verb === 'POST' || verb === 'PUT' || verb === 'PATCH') {
                const formData = new FormData(target);
                const data = Object.fromEntries(formData.entries());

                // Queue mutation
                await this.queueMutation(entity, verb, data);

                // Update local IndexedDB
                if (verb === 'POST') {
                    data.id = Date.now(); // Temporary ID
                }
                await this.applyLocalMutation(entity, verb, data);

                // Trigger success
                this.triggerRefresh(entity, data.id);
            }
        }

        /**
         * Queue mutation for later sync
         */
        async queueMutation(entity, action, data) {
            const tx = this.db.transaction('_pending', 'readwrite');
            const store = tx.objectStore('_pending');

            await store.add({
                entity,
                action,
                data,
                timestamp: Date.now(),
            });

            return new Promise((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Apply mutation to local IndexedDB
         */
        async applyLocalMutation(entity, action, data) {
            const tx = this.db.transaction(entity, 'readwrite');
            const store = tx.objectStore(entity);

            if (action === 'DELETE') {
                await store.delete(data.id);
            } else {
                await store.put(data);
            }

            return new Promise((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Sync pending mutations when back online
         */
        async syncPendingMutations() {
            if (this.syncInProgress) return;
            this.syncInProgress = true;

            try {
                const tx = this.db.transaction('_pending', 'readonly');
                const store = tx.objectStore('_pending');
                const request = store.getAll();

                const pending = await new Promise((resolve, reject) => {
                    request.onsuccess = () => resolve(request.result);
                    request.onerror = () => reject(request.error);
                });

                this.log(`Syncing ${pending.length} pending mutations`);

                for (const mutation of pending) {
                    await this.pushMutation(mutation);
                }

                // Clear pending queue
                await this.clearPending();

                // Re-sync all entities
                await this.initialSync();
            } catch (error) {
                this.error('Failed to sync pending mutations:', error);
            } finally {
                this.syncInProgress = false;
            }
        }

        /**
         * Push a mutation to the server
         */
        async pushMutation(mutation) {
            try {
                const response = await fetch(`/api/sync/${mutation.entity}`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        changes: [{
                            id: mutation.data.id.toString(),
                            action: mutation.action.toLowerCase(),
                            data: mutation.data,
                        }],
                    }),
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }

                this.log('Pushed mutation', mutation);
            } catch (error) {
                this.error('Failed to push mutation:', error);
                throw error;
            }
        }

        /**
         * Clear pending mutations
         */
        async clearPending() {
            const tx = this.db.transaction('_pending', 'readwrite');
            const store = tx.objectStore('_pending');
            await store.clear();

            return new Promise((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Initialize the sync system
         */
        async init() {
            try {
                await this.initIndexedDB();
                await this.initialSync();
                this.connectSSE();
                this.setupOfflineHandlers();

                this.log('RHTMX Sync initialized successfully');

                // Dispatch ready event
                document.dispatchEvent(new Event('rhtmx:sync:ready'));
            } catch (error) {
                this.error('Failed to initialize:', error);
            }
        }
    }

    // Auto-initialize when DOM is ready
    function autoInit() {
        const script = document.currentScript ||
                      document.querySelector('[data-sync-entities]');

        if (!script) {
            console.warn('[RHTMX Sync] No configuration found. Add data-sync-entities attribute to script tag.');
            return;
        }

        const entities = (script.dataset.syncEntities || '').split(',').filter(e => e.trim());
        const conflictStrategy = script.dataset.conflictStrategy || 'last-write-wins';
        const debug = script.dataset.debug === 'true';

        if (entities.length === 0) {
            console.warn('[RHTMX Sync] No entities specified in data-sync-entities');
            return;
        }

        const sync = new RHTMXSync({ entities, conflictStrategy, debug });
        sync.init();

        // Expose to window for manual control
        window.rhtmxSync = sync;
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', autoInit);
    } else {
        autoInit();
    }
})();
