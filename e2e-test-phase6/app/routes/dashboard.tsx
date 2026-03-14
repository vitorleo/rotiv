import { defineRoute } from "@rotiv/sdk";

export default defineRoute({
  path: "/dashboard",

  async loader(ctx) {
    return { title: "Dashboard" };
  },

  component({ data }) {
    return (
      <main>
        <h1>{data.title}</h1>
        <p>Protected dashboard (auth module required)</p>
      </main>
    );
  },
});
