export interface RotivErrorJson {
  code: string;
  message: string;
  file?: string;
  line?: number;
  suggestion?: string;
}

interface StackLine {
  file: string;
  line: number;
}

function parseStackLine(stack: string | undefined): StackLine | null {
  if (!stack) return null;
  // Match lines like: "    at Object.<anonymous> (/path/to/file.ts:12:5)"
  const match = stack.match(/at\s+\S+\s+\((.+?):(\d+):\d+\)/);
  if (match && match[1] && match[2]) {
    return { file: match[1], line: parseInt(match[2], 10) };
  }
  // Simpler form: "    at /path/to/file.ts:12:5"
  const simpleMatch = stack.match(/at\s+(.+?):(\d+):\d+/);
  if (simpleMatch && simpleMatch[1] && simpleMatch[2]) {
    return { file: simpleMatch[1], line: parseInt(simpleMatch[2], 10) };
  }
  return null;
}

export function toRotivError(err: unknown, routeFile: string): RotivErrorJson {
  if (err instanceof Error) {
    const stackLine = parseStackLine(err.stack);
    const result: RotivErrorJson = {
      code: "E_ROUTE_EXEC",
      message: err.message,
      file: stackLine?.file ?? routeFile,
      suggestion: "Check the error message and the stack trace above for details",
    };
    if (stackLine?.line !== undefined) result.line = stackLine.line;
    return result;
  }
  return {
    code: "E_ROUTE_EXEC",
    message: typeof err === "string" ? err : JSON.stringify(err),
    file: routeFile,
  };
}
