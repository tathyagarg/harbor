import { PUBLIC_VERSION } from '$env/static/public';

export function GET() {
  return new Response(JSON.stringify({
    version: PUBLIC_VERSION
  }))
}
