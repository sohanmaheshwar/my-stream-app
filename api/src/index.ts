import { HandleRequest, HttpRequest, HttpResponse, InferencingModels, Llm, Router } from "@fermyon/spin-sdk";

let router = Router();
let decoder = new TextDecoder();

interface UserPrompt {
  id: string,
  content: string,
}

router.post("/api/generate", async (_req, extra) => {
  let body = JSON.parse(decoder.decode(extra.body)) as UserPrompt;
  let response = Llm.infer(InferencingModels.Llama2Chat, body.content, { maxTokens: 50 });
  console.log(response);
  return { status: 200, body: response.text }
});

router.all("*", async () => {
  return { status: 404 }
})

export const handleRequest: HandleRequest = async function(request: HttpRequest): Promise<HttpResponse> {
  return router.handleRequest(request, { body: request.body });
}
