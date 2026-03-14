// EXAMPLE: Dynamic route — handles "/todos/:id".
// The filename uses [id] bracket notation; the path uses :id colon notation.
// ctx.params.id contains the runtime value (e.g. "42" for /todos/42).
import { defineRoute } from "@rotiv/sdk";
import { todos } from "../../models/todo.js";
import { eq } from "@rotiv/orm";

export default defineRoute({
  // EXAMPLE: Dynamic segment in path — :id matches any value at this position.
  // Corresponds to the [id] in the filename app/routes/todos/[id].tsx.
  path: "/todos/:id",

  // EXAMPLE: loader() accesses ctx.params.id for the dynamic segment value.
  // Always validate/coerce dynamic params — they arrive as strings.
  async loader(ctx) {
    const id = Number(ctx.params.id);
    if (isNaN(id)) {
      // EXAMPLE: Throw a Response to short-circuit the loader and return early.
      throw new Response("Not found", { status: 404 });
    }

    // EXAMPLE: Drizzle where clause — eq() is the equals operator.
    // .get() returns a single row or undefined.
    const [todo] = await ctx.db.drizzle
      .select()
      .from(todos)
      .where(eq(todos.id, id))
      .limit(1);

    if (!todo) {
      throw new Response("Not found", { status: 404 });
    }

    return { todo };
  },

  // EXAMPLE: action() handles status toggle (POST with _method=PATCH workaround).
  async action(ctx) {
    const id = Number(ctx.params.id);
    const formData = await ctx.request.formData();
    const newStatus = formData.get("status") === "done" ? "done" : "pending";

    // EXAMPLE: Drizzle update with a where clause.
    await ctx.db.drizzle
      .update(todos)
      .set({ status: newStatus })
      .where(eq(todos.id, id));

    return Response.redirect(`/todos/${id}`, 303);
  },

  // EXAMPLE: component() — data.todo is typed as Todo from the loader return.
  component({ data }) {
    const { todo } = data;
    return (
      <main>
        <a href="/">← Back to list</a>
        <h1>{todo.title}</h1>
        <p>Status: <strong>{todo.status}</strong></p>
        <p>Created: {todo.createdAt}</p>

        {/* EXAMPLE: Form to toggle todo status */}
        <form method="post">
          <input
            type="hidden"
            name="status"
            value={todo.status === "done" ? "pending" : "done"}
          />
          <button type="submit">
            {todo.status === "done" ? "Mark as pending" : "Mark as done"}
          </button>
        </form>
      </main>
    );
  },
});
