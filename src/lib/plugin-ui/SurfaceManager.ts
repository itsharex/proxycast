/**
 * @file Surface 管理器
 * @description 管理插件的 UI Surface 状态
 * @module lib/plugin-ui/SurfaceManager
 */

import type {
  SurfaceId,
  PluginId,
  ComponentId,
  ComponentDef,
  SurfaceState,
  SurfaceStyles,
  ServerMessage,
  DataEntry,
} from "./types";
import { DataStore } from "./DataStore";

/**
 * Surface 管理器
 * 管理所有插件的 UI Surface
 */
export class SurfaceManager {
  private surfaces: Map<SurfaceId, SurfaceState> = new Map();
  private dataStores: Map<SurfaceId, DataStore> = new Map();
  private listeners: Set<(surfaces: Map<SurfaceId, SurfaceState>) => void> =
    new Set();

  /**
   * 处理服务端消息
   */
  processMessage(pluginId: PluginId, message: ServerMessage): void {
    if ("surfaceUpdate" in message) {
      this.handleSurfaceUpdate(
        pluginId,
        message.surfaceUpdate.surfaceId,
        message.surfaceUpdate.components,
      );
    } else if ("dataModelUpdate" in message) {
      this.handleDataModelUpdate(
        message.dataModelUpdate.surfaceId,
        message.dataModelUpdate.path,
        message.dataModelUpdate.contents,
      );
    } else if ("beginRendering" in message) {
      this.handleBeginRendering(
        pluginId,
        message.beginRendering.surfaceId,
        message.beginRendering.root,
        message.beginRendering.styles,
      );
    } else if ("deleteSurface" in message) {
      this.handleDeleteSurface(message.deleteSurface.surfaceId);
    }
  }

  /**
   * 处理 Surface 更新
   */
  private handleSurfaceUpdate(
    pluginId: PluginId,
    surfaceId: SurfaceId,
    components: ComponentDef[],
  ): void {
    let surface = this.surfaces.get(surfaceId);

    if (!surface) {
      // 创建新 Surface
      surface = {
        surfaceId,
        pluginId,
        rootId: null,
        components: new Map(),
        dataModel: {},
        styles: {},
        isReady: false,
      };
      this.surfaces.set(surfaceId, surface);
      this.dataStores.set(surfaceId, new DataStore());
    }

    // 更新组件
    for (const comp of components) {
      surface.components.set(comp.id, comp);
    }

    this.notifyListeners();
  }

  /**
   * 处理数据模型更新
   */
  private handleDataModelUpdate(
    surfaceId: SurfaceId,
    path: string | undefined,
    contents: DataEntry[],
  ): void {
    const surface = this.surfaces.get(surfaceId);
    const dataStore = this.dataStores.get(surfaceId);

    if (!surface || !dataStore) {
      console.warn(`[SurfaceManager] Surface '${surfaceId}' 不存在`);
      return;
    }

    // 更新数据存储
    dataStore.applyUpdate(path, contents);

    // 同步到 surface 状态
    surface.dataModel = dataStore.getData();

    this.notifyListeners();
  }

  /**
   * 处理开始渲染
   */
  private handleBeginRendering(
    pluginId: PluginId,
    surfaceId: SurfaceId,
    rootId: ComponentId,
    styles?: SurfaceStyles,
  ): void {
    let surface = this.surfaces.get(surfaceId);

    if (!surface) {
      // 如果 Surface 不存在，创建一个
      surface = {
        surfaceId,
        pluginId,
        rootId: null,
        components: new Map(),
        dataModel: {},
        styles: {},
        isReady: false,
      };
      this.surfaces.set(surfaceId, surface);
      this.dataStores.set(surfaceId, new DataStore());
    }

    surface.rootId = rootId;
    surface.isReady = true;

    if (styles) {
      surface.styles = { ...surface.styles, ...styles };
    }

    this.notifyListeners();
  }

  /**
   * 处理删除 Surface
   */
  private handleDeleteSurface(surfaceId: SurfaceId): void {
    this.surfaces.delete(surfaceId);
    this.dataStores.delete(surfaceId);
    this.notifyListeners();
  }

  /**
   * 获取 Surface 状态
   */
  getSurface(surfaceId: SurfaceId): SurfaceState | undefined {
    return this.surfaces.get(surfaceId);
  }

  /**
   * 获取插件的所有 Surface
   */
  getSurfacesByPlugin(pluginId: PluginId): SurfaceState[] {
    return Array.from(this.surfaces.values()).filter(
      (s) => s.pluginId === pluginId,
    );
  }

  /**
   * 获取所有 Surface
   */
  getAllSurfaces(): Map<SurfaceId, SurfaceState> {
    return new Map(this.surfaces);
  }

  /**
   * 获取 Surface 的数据存储
   */
  getDataStore(surfaceId: SurfaceId): DataStore | undefined {
    return this.dataStores.get(surfaceId);
  }

  /**
   * 订阅 Surface 变化
   */
  subscribe(
    listener: (surfaces: Map<SurfaceId, SurfaceState>) => void,
  ): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * 清理插件的所有 Surface
   */
  clearPlugin(pluginId: PluginId): void {
    for (const [surfaceId, surface] of this.surfaces) {
      if (surface.pluginId === pluginId) {
        this.surfaces.delete(surfaceId);
        this.dataStores.delete(surfaceId);
      }
    }
    this.notifyListeners();
  }

  /**
   * 清理所有 Surface
   */
  clear(): void {
    this.surfaces.clear();
    this.dataStores.clear();
    this.notifyListeners();
  }

  private notifyListeners(): void {
    const snapshot = new Map(this.surfaces);
    for (const listener of this.listeners) {
      listener(snapshot);
    }
  }
}

/**
 * 全局 Surface 管理器实例
 */
export const surfaceManager = new SurfaceManager();
