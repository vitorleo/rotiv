import { defineRoute } from "@rotiv/sdk";

export default defineRoute({
  path: "/",
  async loader() {
    return { message: "Hello from {{project_name}}!" };
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
