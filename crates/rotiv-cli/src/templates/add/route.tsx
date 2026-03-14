// FRAMEWORK: Route file — defines a single URL endpoint.
// Export default must be defineRoute({ path, loader?, action?, component? }).
// loader() runs server-side before the component renders; its return value becomes `data`.
// action() handles mutations (POST, PUT, PATCH, DELETE).
// component() receives { data } typed from the loader return type.
import { defineRoute } from "@rotiv/sdk";

export default defineRoute({
  // FRAMEWORK: path must match the file's location under app/routes/.
  // File: app/routes/{{route_file_path}}.tsx → path: "{{route_path}}"
  // Dynamic segments use [param] in the filename → :param in the path.
  // Example: app/routes/users/[id].tsx → path: "/users/:id"
  path: "{{route_path}}",

  // FRAMEWORK: loader runs on the server for every GET request to this route.
  // ctx.request — the raw Request object
  // ctx.params  — dynamic path params (e.g. { id: "42" } for /users/:id)
  // ctx.searchParams — URL query params
  // ctx.db — database connection (RotivDb); use ctx.db.drizzle for type-safe queries
  // Return any plain serializable value — it becomes `data` in component().
  async loader(ctx) {
    return { message: "Hello from {{route_path}}" };
  },

  // FRAMEWORK: component receives { data } typed from loader's return type.
  // Use JSX — no React import needed. Uses @rotiv/jsx-runtime automatically.
  component({ data }) {
    return (
      <main>
        <h1>{{route_path}}</h1>
        <p>{data.message}</p>
      </main>
    );
  },
});
