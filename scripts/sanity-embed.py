import json, math, urllib.request
URL = "http://lain:8081/v1/embeddings"
q = "Instruct: Given a web search query, retrieve relevant passages that answer the query\nQuery: how do I ban IPs that brute-force ssh"
docs = ["fail2ban monitors auth logs and adds firewall rules to block repeated failed logins.",
        "Sourdough starter needs regular feeding with flour and water."]
body = json.dumps({"input": [q] + docs}).encode()
req = urllib.request.Request(URL, body, {"Content-Type": "application/json"})
data = [d["embedding"] for d in json.load(urllib.request.urlopen(req))["data"]]
cos = lambda a, b: sum(x*y for x, y in zip(a, b)) / (math.hypot(*a) * math.hypot(*b))
print(len(data), len(data[0]))   # expect: 3 1024
print("relevant:  ", round(cos(data[0], data[1]), 3))
print("irrelevant:", round(cos(data[0], data[2]), 3))
