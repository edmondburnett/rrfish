import json, time, urllib.request, uuid
base = "fail2ban watches auth logs and bans hosts with repeated failures. " * 30
inputs = [f"{uuid.uuid4()} {base}" for _ in range(64)]   # unique prefix defeats the cache
body = json.dumps({"input": inputs}).encode()
req = urllib.request.Request("http://lain:8081/v1/embeddings", body, {"Content-Type": "application/json"})
t = time.time(); r = json.load(urllib.request.urlopen(req)); dt = time.time() - t
toks = r["usage"]["prompt_tokens"]
print(f"{toks} tokens in {dt:.2f}s -> {toks/dt:.0f} tok/s")
