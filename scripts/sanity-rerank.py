import json, urllib.request

URL = "http://lain:8082/v1/rerank"
body = json.dumps({
    "query": "how do I ban IPs that brute-force ssh",
    "documents": [
        "fail2ban monitors auth logs and adds firewall rules to block repeated failed logins.",
        "Sourdough starter needs regular feeding with flour and water.",
        "To block IPs that repeatedly fail ssh logins, enable the fail2ban sshd jail.",
    ],
}).encode()
req = urllib.request.Request(URL, body, {"Content-Type": "application/json"})
for r in json.load(urllib.request.urlopen(req))["results"]:
    print(r["index"], round(r["relevance_score"], 4))
