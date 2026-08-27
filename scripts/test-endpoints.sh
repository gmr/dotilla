#!/usr/bin/env zsh

HOST="${1:-localhost}"
PORT="${2:-6465}"

QUERY='MATCH (f:Foo)-[b:BAR]->(z:Baz) RETURN f, b, z'


http_req() {
	local endpoint="$1"
	shift
	local response http_status body

	response=$(curl -s -w $'\n%{http_code}' "$@" "http://${HOST}:${PORT}/${endpoint}")
	http_status="${response##*$'\n'}"
	body="${response%$'\n'*}"

	printf "HTTP %s\n" "$http_status"
	if [[ "${=body}" != "" ]]; then
		printf '%s\n' "${body}"
	fi
}

http_req_json() {
	local endpoint="$1"
	shift
	local response http_status body

	response=$(curl -s -w $'\n%{http_code}' "$@" "http://${HOST}:${PORT}/${endpoint}")
	http_status="${response##*$'\n'}"
	body="${response%$'\n'*}"

	printf "HTTP %s\n" "$http_status"
	if [[ "${=body}" != "" ]]; then
		printf '%s\n' "${body}" | jq .
	fi
}

http_head() {
   printf "\nHEAD /${1}\n"
   http_req_json $1 -I -o /dev/null
}

http_get() {
	printf "\nGET /${1}\n"
	http_req_json $1
}

http_post() {
	printf "\nPOST /${1}\n"
	http_req_json $1 --data "$2" -H "Content-Type: application/json"
}


http_post_text() {
	printf "\nPOST /${1}\n"
	http_req $1 --data "$2" -o post.dot -H "Content-Type: text/plain"
}

http_put() {
	printf "\nPUT /${1}\n"
	http_req_json $1 --data "$2" -X PUT -H "Content-Type: application/json"
}

http_delete() {
	printf "\nDELETE /${1}\n"
	http_req_json $1 -X DELETE
}

# DBs and Info
http_get "_all_namespaces"
http_get "_db_info"

# Invalid database
http_head "~invalid~"
http_get "~invalid~"
http_put "~invalid~"

http_delete "test"
http_head "test"
http_get "test"
http_put "test" "{}"
http_head "test"
http_get "test"
http_put "test" "{}"

http_get "_all_namespaces"
http_get "_db_info"

http_delete "test"
http_head "test"
http_get "test"

http_post_text "test" "MATCH (a:Foo)-[b:BAR]->(c:Baz) RETURN a, b, c"
