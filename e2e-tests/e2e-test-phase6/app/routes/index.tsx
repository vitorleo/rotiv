import { defineRoute } from "@rotiv/sdk";

export default defineRoute({
  path: "/",

  async loader(ctx) {
    return { message: "Welcome to e2e-test-phase6" };
  },

  component({ data }) {
    return (
      <main>
        <h1>Phase 6 Test</h1>
        <p>{data.message}</p>
      </main>
    );
  },
});
