run-greenmail:
  docker run -t -i -p 3025:3025 -p 3110:3110 -p 3143:3143 -p 3465:3465 -p 3993:3993 -p 3995:3995 -p 8090:8080 -e GREENMAIL_OPTS='-Dgreenmail.verbose -Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.auth.disabled' greenmail/standalone:2.1.8 

# Send an email as user 'foo' with password 'a'
send-email recipient="user@example.com" subject="Test Mail" body="Hello from foo":
    echo "{{body}}" | mail -s "{{subject}}" --mailer="smtp://foo:a@localhost:3025" -r foo@example.com {{recipient}}

cleanup-greenmail:
  docker ps -a --format '{{"{{"}}.ID}} {{"{{"}}.Image}}' | grep 'greenmail' | awk '{print $1}' | xargs docker rm -f

# Create a folder on an IMAP server
# Usage: just create-folder <folder-name>
# For subfolders, use parent/child format like: just create-folder "INBOX/tests1"
create-folder folder server="localhost" port="3143" user="user@example.com" password="a":
  python3 -c "import imaplib; c = imaplib.IMAP4('{{server}}', {{port}}); c.login('{{user}}', '{{password}}'); folder_name = '{{folder}}'; folder_name = folder_name.replace('/', '.'); result = c.create(folder_name); print('Created folder:', folder_name if result[0] == 'OK' else 'Failed:', result); c.logout()"

