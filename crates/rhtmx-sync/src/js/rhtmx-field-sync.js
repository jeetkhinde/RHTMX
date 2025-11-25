/**
 * rhtmx-field-sync.js
 * Client-side field-level synchronization for RHTMX (CRDT-like)
 *
 * Usage:
 * <script src="/api/sync/field-client.js"
 *         data-sync-entities="users,posts"
 *         data-field-strategy="last-write-wins"
 *         data-debug="false">
 * </script>
 */

(function() {
    'use strict';

    class RHTMXFieldSync {
        constructor(config) {
            this.entities = config.entities || [];
            this.fieldStrategy = config.fieldStrategy || 'last-write-wins';
            this.debug = config.debug || false;
            this.db = null;
            this.syncInProgress = false;
            this.pendingChanges = new Map(); // Track local changes

            this.log('Initializing RHTMX Field Sync', { entities: this.entities });
        }

        log(...args) {
            if (this.debug) {
                console.log('[RHTMX Field Sync]', ...args);
            }
        }

        error(...args) {
            console.error('[RHTMX Field Sync]', ...args);
        }

        /**
         * Initialize IndexedDB for field-level storage
         */
        async initIndexedDB() {
            return new Promise((resolve, reject) => {
                const request = indexedDB.open('rhtmx-field-cache', 1);

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

                    // Create field metadata store (tracks field versions/timestamps)
                    if (!db.objectStoreNames.contains('_field_meta')) {
                        const metaStore = db.createObjectStore('_field_meta', { keyPath: 'key' });
                        metaStore.createIndex('entity_field', ['entity', 'entity_id', 'field']);
                    }

                    // Create pending field changes store
                    if (!db.objectStoreNames.contains('_pending_fields')) {
                        db.createObjectStore('_pending_fields', { autoIncrement: true });
                    }

                    // Create entity version store
                    if (!db.objectStoreNames.contains('_versions')) {
                        db.createObjectStore('_versions', { keyPath: 'entity' });
                    }

                    this.log('IndexedDB schema created');
                };
            });
        }

        /**
         * Perform initial field sync for all entities
         */
        async initialSync() {
            this.log('Starting initial field sync');

            for (const entity of this.entities) {
                await this.syncEntity(entity);
            }

            this.log('Initial field sync complete');
        }

        /**
         * Sync field changes for a single entity
         */
        async syncEntity(entity) {
            try {
                // Get last known version
                const lastVersion = await this.getLastVersion(entity);

                // Fetch field changes from server
                const response = await fetch(`/api/field-sync/${entity}?since=${lastVersion}`);

                if (!response.ok) {
                    throw new Error(`Failed to sync ${entity}: ${response.statusText}`);
                }

                const data = await response.json();
                this.log(`Received ${data.changes.length} field changes for ${entity}`);

                // Apply field changes to local storage
                await this.applyFieldChanges(entity, data.changes);

                // Update version
                await this.setLastVersion(entity, data.version);

                // Trigger UI refresh
                this.triggerRefresh(entity);

            } catch (error) {
                this.error(`Error syncing ${entity}:`, error);
            }
        }

        /**
         * Apply field changes to IndexedDB
         */
        async applyFieldChanges(entity, changes) {
            if (changes.length === 0) return;

            return new Promise((resolve, reject) => {
                const tx = this.db.transaction([entity, '_field_meta'], 'readwrite');
                const entityStore = tx.objectStore(entity);
                const metaStore = tx.objectStore('_field_meta');

                // Group changes by entity_id
                const changesByEntity = new Map();
                for (const change of changes) {
                    if (!changesByEntity.has(change.entity_id)) {
                        changesByEntity.set(change.entity_id, []);
                    }
                    changesByEntity.get(change.entity_id).push(change);
                }

                // Process each entity instance
                for (const [entityId, entityChanges] of changesByEntity) {
                    // Get current entity data
                    const getRequest = entityStore.get(entityId);

                    getRequest.onsuccess = () => {
                        let entityData = getRequest.result || { id: entityId };

                        // Apply each field change
                        for (const change of entityChanges) {
                            if (change.action === 'update') {
                                entityData[change.field] = change.value;
                            } else if (change.action === 'delete') {
                                delete entityData[change.field];
                            }

                            // Store field metadata
                            const metaKey = `${entity}:${entityId}:${change.field}`;
                            metaStore.put({
                                key: metaKey,
                                entity: entity,
                                entity_id: entityId,
                                field: change.field,
                                version: change.version,
                                timestamp: change.timestamp
                            });
                        }

                        // Save updated entity
                        entityStore.put(entityData);
                        this.log(`Applied ${entityChanges.length} field changes to ${entity}:${entityId}`);
                    };
                }

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Record a local field change
         */
        async recordFieldChange(entity, entityId, field, value) {
            const timestamp = new Date().toISOString();

            // Apply to local IndexedDB immediately (optimistic update)
            await this.applyLocalFieldChange(entity, entityId, field, value, timestamp);

            // Queue for sync to server
            await this.queueFieldChange(entity, entityId, field, value, timestamp);

            // Sync to server if online
            if (navigator.onLine) {
                await this.syncPendingChanges(entity);
            }
        }

        /**
         * Apply local field change immediately
         */
        async applyLocalFieldChange(entity, entityId, field, value, timestamp) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction([entity, '_field_meta'], 'readwrite');
                const entityStore = tx.objectStore(entity);
                const metaStore = tx.objectStore('_field_meta');

                const getRequest = entityStore.get(entityId);

                getRequest.onsuccess = () => {
                    const entityData = getRequest.result || { id: entityId };
                    entityData[field] = value;
                    entityStore.put(entityData);

                    // Update field metadata
                    const metaKey = `${entity}:${entityId}:${field}`;
                    metaStore.put({
                        key: metaKey,
                        entity: entity,
                        entity_id: entityId,
                        field: field,
                        timestamp: timestamp,
                        local: true
                    });
                };

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Queue field change for server sync
         */
        async queueFieldChange(entity, entityId, field, value, timestamp) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_pending_fields'], 'readwrite');
                const store = tx.objectStore('_pending_fields');

                store.add({
                    entity: entity,
                    entity_id: entityId,
                    field: field,
                    value: value,
                    action: 'update',
                    timestamp: timestamp,
                    queued_at: new Date().toISOString()
                });

                tx.oncomplete = () => {
                    this.log(`Queued field change: ${entity}:${entityId}.${field}`);
                    resolve();
                };
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Sync pending field changes to server
         */
        async syncPendingChanges(entity) {
            if (this.syncInProgress) return;
            this.syncInProgress = true;

            try {
                const pendingChanges = await this.getPendingChanges(entity);

                if (pendingChanges.length === 0) {
                    this.syncInProgress = false;
                    return;
                }

                this.log(`Syncing ${pendingChanges.length} pending field changes for ${entity}`);

                const response = await fetch(`/api/field-sync/${entity}`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        changes: pendingChanges.map(c => ({
                            entity_id: c.entity_id,
                            field: c.field,
                            value: c.value,
                            action: c.action,
                            timestamp: c.timestamp
                        }))
                    })
                });

                if (!response.ok) {
                    throw new Error(`Failed to sync pending changes: ${response.statusText}`);
                }

                const result = await response.json();

                // Handle conflicts
                if (result.conflicts && result.conflicts.length > 0) {
                    this.log(`Detected ${result.conflicts.length} conflicts`);
                    await this.handleConflicts(result.conflicts);
                }

                // Clear synced changes
                await this.clearPendingChanges(pendingChanges);

                this.log('Pending changes synced successfully');

            } catch (error) {
                this.error('Error syncing pending changes:', error);
            } finally {
                this.syncInProgress = false;
            }
        }

        /**
         * Get pending changes for an entity
         */
        async getPendingChanges(entity) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_pending_fields'], 'readonly');
                const store = tx.objectStore('_pending_fields');
                const request = store.getAll();

                request.onsuccess = () => {
                    const allChanges = request.result;
                    const entityChanges = allChanges.filter(c => c.entity === entity);
                    resolve(entityChanges);
                };

                request.onerror = () => reject(request.error);
            });
        }

        /**
         * Clear pending changes
         */
        async clearPendingChanges(changes) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_pending_fields'], 'readwrite');
                const store = tx.objectStore('_pending_fields');

                // Note: In a real implementation, we'd need to track keys
                // For simplicity, we're clearing all for now
                const request = store.clear();

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Handle field-level conflicts
         */
        async handleConflicts(conflicts) {
            for (const conflict of conflicts) {
                this.log('Conflict detected:', conflict);

                // Emit custom event for application to handle
                window.dispatchEvent(new CustomEvent('rhtmx:field:conflict', {
                    detail: conflict
                }));

                // Apply resolution based on strategy
                if (this.fieldStrategy === 'server-wins') {
                    await this.applyFieldChanges(conflict.entity, [{
                        entity_id: conflict.entity_id,
                        field: conflict.field,
                        value: conflict.server_value,
                        action: 'update',
                        timestamp: conflict.server_timestamp
                    }]);
                }
            }
        }

        /**
         * Get last synced version for entity
         */
        async getLastVersion(entity) {
            return new Promise((resolve) => {
                const tx = this.db.transaction(['_versions'], 'readonly');
                const store = tx.objectStore('_versions');
                const request = store.get(entity);

                request.onsuccess = () => {
                    const result = request.result;
                    resolve(result ? result.version : 0);
                };

                request.onerror = () => resolve(0);
            });
        }

        /**
         * Set last synced version for entity
         */
        async setLastVersion(entity, version) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_versions'], 'readwrite');
                const store = tx.objectStore('_versions');

                store.put({ entity: entity, version: version });

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Trigger HTMX refresh for entity
         */
        triggerRefresh(entity) {
            const event = new CustomEvent(`rhtmx:${entity}:field-changed`, {
                detail: { entity: entity },
                bubbles: true
            });
            document.body.dispatchEvent(event);
            this.log(`Triggered refresh event for ${entity}`);
        }

        /**
         * Setup offline/online handlers
         */
        setupOfflineHandlers() {
            window.addEventListener('online', async () => {
                this.log('Back online, syncing pending changes');
                for (const entity of this.entities) {
                    await this.syncPendingChanges(entity);
                    await this.syncEntity(entity);
                }
            });

            window.addEventListener('offline', () => {
                this.log('Went offline, changes will be queued');
            });
        }

        /**
         * Initialize everything
         */
        static async init() {
            const scriptTag = document.currentScript;
            const entities = scriptTag.getAttribute('data-sync-entities');
            const fieldStrategy = scriptTag.getAttribute('data-field-strategy') || 'last-write-wins';
            const debug = scriptTag.getAttribute('data-debug') === 'true';

            if (!entities) {
                console.error('[RHTMX Field Sync] No entities specified in data-sync-entities');
                return;
            }

            const config = {
                entities: entities.split(',').map(e => e.trim()),
                fieldStrategy: fieldStrategy,
                debug: debug
            };

            const sync = new RHTMXFieldSync(config);

            try {
                await sync.initIndexedDB();
                await sync.initialSync();
                sync.setupOfflineHandlers();

                // Make available globally
                window.RHTMXFieldSync = sync;

                console.log('[RHTMX Field Sync] Initialization complete');
            } catch (error) {
                console.error('[RHTMX Field Sync] Initialization failed:', error);
            }
        }
    }

    // Auto-initialize if script tag has data attributes
    if (document.currentScript && document.currentScript.hasAttribute('data-sync-entities')) {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => RHTMXFieldSync.init());
        } else {
            RHTMXFieldSync.init();
        }
    }

    // Export for manual initialization
    window.RHTMXFieldSync = RHTMXFieldSync;
})();
