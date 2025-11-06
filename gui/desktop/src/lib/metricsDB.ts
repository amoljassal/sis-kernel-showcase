/**
 * IndexedDB storage for metrics - M5 Enhancement
 *
 * Provides persistent storage for metrics data across sessions
 * with efficient querying and automatic cleanup.
 */

import { MetricPoint } from './api';

const DB_NAME = 'SisKernelMetrics';
const DB_VERSION = 1;
const STORE_NAME = 'metrics';
const MAX_AGE_MS = 24 * 60 * 60 * 1000; // 24 hours

export interface MetricSeries {
  name: string;
  points: MetricPoint[];
  lastUpdated: number;
}

class MetricsDB {
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void> | null = null;

  async init(): Promise<void> {
    if (this.db) return;
    if (this.initPromise) return this.initPromise;

    this.initPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Create object store if it doesn't exist
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          const store = db.createObjectStore(STORE_NAME, { keyPath: 'name' });
          store.createIndex('lastUpdated', 'lastUpdated', { unique: false });
        }
      };
    });

    return this.initPromise;
  }

  async saveSeries(name: string, points: MetricPoint[]): Promise<void> {
    await this.init();
    if (!this.db) throw new Error('Database not initialized');

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);

      const data: MetricSeries = {
        name,
        points,
        lastUpdated: Date.now(),
      };

      const request = store.put(data);
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async getSeries(name: string): Promise<MetricSeries | null> {
    await this.init();
    if (!this.db) throw new Error('Database not initialized');

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, 'readonly');
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(name);

      request.onsuccess = () => {
        const result = request.result as MetricSeries | undefined;
        if (!result) {
          resolve(null);
          return;
        }

        // Check if data is stale
        const age = Date.now() - result.lastUpdated;
        if (age > MAX_AGE_MS) {
          // Delete stale data
          this.deleteSeries(name);
          resolve(null);
        } else {
          resolve(result);
        }
      };
      request.onerror = () => reject(request.error);
    });
  }

  async getAllSeries(): Promise<MetricSeries[]> {
    await this.init();
    if (!this.db) throw new Error('Database not initialized');

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, 'readonly');
      const store = tx.objectStore(STORE_NAME);
      const request = store.getAll();

      request.onsuccess = () => {
        const results = (request.result as MetricSeries[]) || [];
        // Filter out stale data
        const now = Date.now();
        const fresh = results.filter((series) => now - series.lastUpdated <= MAX_AGE_MS);
        resolve(fresh);
      };
      request.onerror = () => reject(request.error);
    });
  }

  async deleteSeries(name: string): Promise<void> {
    await this.init();
    if (!this.db) throw new Error('Database not initialized');

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      const request = store.delete(name);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async clearAll(): Promise<void> {
    await this.init();
    if (!this.db) throw new Error('Database not initialized');

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      const request = store.clear();

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async cleanup(): Promise<number> {
    await this.init();
    if (!this.db) throw new Error('Database not initialized');

    const allSeries = await this.getAllSeries();
    const now = Date.now();
    let deletedCount = 0;

    for (const series of allSeries) {
      if (now - series.lastUpdated > MAX_AGE_MS) {
        await this.deleteSeries(series.name);
        deletedCount++;
      }
    }

    return deletedCount;
  }
}

export const metricsDB = new MetricsDB();
