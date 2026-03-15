import type { RequestHandler } from "@sveltejs/kit";

export const prerender = true;

export const GET: RequestHandler = async () => {
  return new Response(
    JSON.stringify({
      version: "v0.0.1",
    }),
    {
      headers: {
        "Content-Type": "application/json",
      },
    },
  )
}
