/**
 * Spec version string. Currently "1".
 */
export type SpecVersion = "1";

export interface RouteEntry {
  path: string;
  file: string;
  methods?: string[];
}

export interface ModelEntry {
  name: string;
  file: string;
}

export type ModuleTier = "primitive" | "slot" | "escape_hatch";

export interface ModuleEntry {
  name: string;
  version: string;
  description?: string;
  provides?: string[];
  requires?: string[];
  configures?: string[];
  tier?: ModuleTier;
  entry?: string;
  test?: string;
}

export interface SpecConventions {
  routes_dir: string;
  models_dir: string;
  components_dir: string;
}

/**
 * The `.rotiv/spec.json` file structure.
 * This is the single source of truth for a Rotiv project's metadata.
 */
export interface RotivSpec {
  $schema?: string;
  version: SpecVersion;
  framework_version: string;
  project: {
    name: string;
    created_at: string;
  };
  routes: RouteEntry[];
  models: ModelEntry[];
  modules: ModuleEntry[];
  conventions?: SpecConventions;
}
