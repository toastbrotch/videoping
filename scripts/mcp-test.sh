#!/bin/sh
# usage: VIDEOPING_SELFTEST=<dir> VIDEOPING_SELFTEST_MCP=7467 VIDEOPING_SELFTEST_STAY=14 videoping &  then: scripts/mcp-test.sh 7467 <outdir>
# MCP calls against a running videoping (port $1), output to $2
P=$1; O=$2
call() { curl -s -X POST -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -d "$1" http://127.0.0.1:$P/mcp; echo; }
for i in $(seq 1 40); do curl -s -o /dev/null http://127.0.0.1:$P/mcp -X POST -d '{}' && break; sleep 0.25; done
call '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}' > $O/init.json
curl -s -o /dev/null -w "notification -> %{http_code}\n" -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' http://127.0.0.1:$P/mcp
call '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' > $O/tools.json
call '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"state","arguments":{}}}' > $O/state.json
call '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"set","arguments":{"address":"/layer/1/mixtype","value":"iLk"}}}' > $O/set.json
call '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get","arguments":{"addresses":["/layer/1/mixtype","/slot/9/fx/1/param/1","/layer/9/mix"]}}}' > $O/get.json
call '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"list_files","arguments":{"kind":"effect","filter":"rutt"}}}' > $O/files.json
call '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"shader_info","arguments":{"path":"isf-effect/rutt-etra.fs"}}}' > $O/info.json
call '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"add_effect","arguments":{"path":"isf-effect/rutt-etra.fs","target":"output"}}}' > $O/addfx.json
call '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"load_file","arguments":{"path":"isf-generator/fractal-terrain.fs","slot":5}}}' > $O/load.json
call '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"list_addresses","arguments":{"filter":"out/fx"}}}' > $O/addr.json
sleep 0.5
call '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"frame","arguments":{"source":"output","width":480}}}' > $O/frame.json
call '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"shader_errors","arguments":{}}}' > $O/errors.json
call '{"jsonrpc":"2.0","id":13,"method":"nope"}' > $O/unknown.json
curl -s -o /dev/null -w "foreign origin -> %{http_code}\n" -X POST -H 'Origin: http://evil.example' -d '{"jsonrpc":"2.0","id":1,"method":"ping"}' http://127.0.0.1:$P/mcp
curl -s -o /dev/null -w "GET -> %{http_code}\n" http://127.0.0.1:$P/mcp
