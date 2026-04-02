import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
  const FILE_URL = "https://raw.githubusercontent.com/tathyagarg/harbor/refs/heads/main/.github/lines.json";

  const response = await fetch(FILE_URL);
  const data: { lines: { total: number }, file_count: number } = await response.json();

  return data;
};
