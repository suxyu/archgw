POST http://localhost:12000/v1/chat/completions
Content-Type: application/json

{
  "messages": [
    {
      "role": "user",
      "content": "hi"
    }
  ],
  "model": "none",
  "stream": true
}
HTTP 200
[Asserts]
header "content-type" matches /text\/event-stream/
body matches /^data: .*?gpt-4.1.*?\n/
