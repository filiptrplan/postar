run-greenmail:
  docker run -t -i -p 3025:3025 -p 3110:3110 -p 3143:3143 -p 3465:3465 -p 3993:3993 -p 3995:3995 -p 8090:8080 greenmail/standalone:2.1.8

# Send an email as user 'foo' with password 'a'
send-email recipient="user@example.com" subject="Test Mail" body="Hello from foo":
    echo "{{body}}" | mail -s "{{subject}}" --mailer="smtp://foo:a@localhost:3025" -r foo@example.com {{recipient}}

cleanup-greenmail:
  docker ps -a --format '{{"{{"}}.ID}} {{"{{"}}.Image}}' | grep 'greenmail' | awk '{print $1}' | xargs docker rm -f


