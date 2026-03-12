/**
 * Top-level project configuration.
 * Used in `rotiv.config.ts` at the project root.
 */
export interface ProjectConfig {
  /** Project name, defaults to package.json name. */
  name: string;
  /** Port for the dev server. Default: 3000. */
  port?: number;
  /** Hostname for the dev server. Default: "localhost". */
  host?: string;
  /** Directory for compiled output. Default: "dist". */
  outDir?: string;
}
