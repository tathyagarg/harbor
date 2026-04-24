import { CORE_DATA } from "$lib";
import { redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

export const load: PageLoad = ({ params }) => {
  if (CORE_DATA[params.name] === undefined) {
    return redirect(302, '/');
  }

  return CORE_DATA[params.name]
}
