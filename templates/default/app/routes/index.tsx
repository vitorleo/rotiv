import { defineRoute } from "@rotiv/sdk";

// FRAMEWORK: Phase 2 — component() must return an HTML string.
// JSX syntax (<h1>) will be supported in Phase 3 (SWC compiler).
export default defineRoute({
  path: "/",
  async loader() {
    return { message: "Hello from {{project_name}}!" };
  },
  component({ data }) {
    return `<main>
  <h1>${data.message}</h1>
  <p>Edit <code>app/routes/index.tsx</code> to get started.</p>
</main>`;
  },
});
