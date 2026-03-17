import type { PageLoad } from './$types';
export const prerender = true;

export const load: PageLoad = async ({ fetch }) => {
  const res = await fetch('https://api.github.com/repos/tathyagarg/harbor/contents/.github/lines.json');

  if (!res.ok) {
    throw new Error('Failed to fetch data');
  }

  const data = await res.json();
  const content = JSON.parse(atob(data.content));

  return {
    total: content.total,
    modules: {
      css: content.ALIASES.css,
      js: content.ALIASES.js,
      html: content.ALIASES.html,
      http: content.ALIASES.http,
      font: content.ALIASES.font,
      render: content.ALIASES.render,
    }
  };
};
