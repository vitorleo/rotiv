// EXAMPLE: Index route — the root page at "/".
// This route renders a todo list and handles adding new todos.
//
// Pattern: loader() fetches data server-side; action() handles form submissions;
// component() renders the UI with type-safe access to loader data.
import { defineRoute } from "@rotiv/sdk";
import { todos } from "../models/todo.js";

export default defineRoute({
  // EXAMPLE: path must match the file's location under app/routes/.
  // "index" maps to "/" — this is the root route.
  path: "/",

  // EXAMPLE: loader() runs on the server before the component renders.
  // ctx.db.drizzle gives you a fully typed Drizzle ORM instance.
  // Return any serializable value — it becomes the `data` prop in component().
  async loader(ctx) {
    // EXAMPLE: Drizzle query — .select() returns an array of typed rows.
    // `todos` is the raw Drizzle table exported from the model file.
    const items = await ctx.db.drizzle.select().from(todos).orderBy(todos.createdAt);
    return { items };
  },

  // EXAMPLE: action() handles POST/PUT/PATCH/DELETE requests to this route.
  // It receives the same ctx as loader(), plus ctx.request for form data.
  // Return a redirect or a plain object (shown as JSON for non-HTML clients).
  async action(ctx) {
    const formData = await ctx.request.formData();
    const title = String(formData.get("title") ?? "").trim();

    if (!title) {
      // EXAMPLE: Return validation errors as plain objects.
      // The framework serializes this as JSON for API clients.
      return { error: "Title is required" };
    }

    // EXAMPLE: Drizzle insert — pass a NewTodo-typed object.
    await ctx.db.drizzle.insert(todos).values({ title });

    // EXAMPLE: Redirect after successful mutation (Post/Redirect/Get pattern).
    return Response.redirect("/", 303);
  },

  // EXAMPLE: component() receives { data } typed from the loader's return type.
  // No React import needed — uses @rotiv/jsx-runtime automatically.
  component({ data }) {
    return (
      <main>
        <h1>Todo List</h1>

        {/* EXAMPLE: Render loader data — data.items is typed as Todo[] */}
        <ul>
          {data.items.map((todo) => (
            <li key={todo.id}>
              {/* EXAMPLE: Link to the detail route with dynamic segment */}
              <a href={`/todos/${todo.id}`}>
                {todo.status === "done" ? <s>{todo.title}</s> : todo.title}
              </a>
              <span> [{todo.status}]</span>
            </li>
          ))}
        </ul>

        {/* EXAMPLE: HTML form — submits POST to this route's action() */}
        <form method="post">
          <input name="title" type="text" placeholder="New todo..." required />
          <button type="submit">Add</button>
        </form>
      </main>
    );
  },
});
