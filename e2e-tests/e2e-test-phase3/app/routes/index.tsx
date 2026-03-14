import { defineRoute } from "@rotiv/sdk";

// FRAMEWORK: Component returns JSX, compiled by @swc/core to @rotiv/jsx-runtime
// calls during `rotiv dev`. Rendering is server-side only in Phase 3.
// Client-side interactivity (signals, DOM binding) arrives in Phase 4.
export default defineRoute({
  path: "/",
  async loader() {
    return { message: "Hello from e2e-test-phase3!" };
  },
  component({ data }) {
    return (
      <main>
        <h1>{data.message}</h1>
        <p>Edit <code>app/routes/index.tsx</code> to get started.</p>
      </main>
    );
  },
});
