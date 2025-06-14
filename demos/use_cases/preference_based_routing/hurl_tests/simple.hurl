POST http://localhost:12000/v1/chat/completions
Content-Type: application/json

{
  "messages": [
    {
      "role": "user",
      "content": "hi"
    }
  ],
  "model": "none"
}
HTTP 200
[Asserts]
header "content-type" == "application/json"
jsonpath "$.model" matches /^gpt-4.1/
jsonpath "$.usage" != null
jsonpath "$.choices[0].message.content" != null
jsonpath "$.choices[0].message.role" == "assistant"
