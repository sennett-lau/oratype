// The site on Cloudflare Workers (site/wrangler.jsonc): serves the built page (site/dist), and
// answers the old domain with a permanent redirect to the same page on the new one, so links and
// search rankings move over (plan `rename-oratype`, web.md).

interface Env {
  ASSETS: { fetch(request: Request): Promise<Response> }
}

const OLD_HOST = 'typelite.sennettlau.me'
const NEW_HOST = 'oratype.sennettlau.me'

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url)
    if (url.hostname === OLD_HOST) {
      url.protocol = 'https:'
      url.hostname = NEW_HOST
      return Response.redirect(url.toString(), 301)
    }
    return env.ASSETS.fetch(request)
  },
}
