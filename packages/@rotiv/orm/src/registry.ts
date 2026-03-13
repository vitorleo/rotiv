import type { ModelDefinition, ModelRegistry } from "./types.js";

class InMemoryModelRegistry implements ModelRegistry {
  private readonly models = new Map<string, ModelDefinition<unknown>>();

  register(model: ModelDefinition<unknown>): void {
    this.models.set(model._name, model);
  }

  get(name: string): ModelDefinition<unknown> | undefined {
    return this.models.get(name);
  }

  getAll(): ModelDefinition<unknown>[] {
    return Array.from(this.models.values());
  }
}

/** Global singleton populated by every `defineModel()` call at import time. */
export const globalModelRegistry: ModelRegistry = new InMemoryModelRegistry();
