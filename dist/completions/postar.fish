# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_postar_global_optspecs
	string join \n c/config= r/rules= log= s/server= db= polling-delay= check dry-run h/help V/version
end

function __fish_postar_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_postar_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_postar_using_subcommand
	set -l cmd (__fish_postar_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c postar -n "__fish_postar_needs_command" -s c -l config -d 'Path to the TOML config file' -r -F
complete -c postar -n "__fish_postar_needs_command" -s r -l rules -d 'Path to the PTAR rules file' -r -F
complete -c postar -n "__fish_postar_needs_command" -l log -d 'The logging level' -r -f -a "off\t''
error\t''
warn\t''
info\t''
debug\t''
trace\t''"
complete -c postar -n "__fish_postar_needs_command" -s s -l server -d 'The server that postar connects to' -r -f -a "(__fish_print_hostnames)"
complete -c postar -n "__fish_postar_needs_command" -l db -d 'Path to the persistent database. Ordinary users should not change this option' -r -F
complete -c postar -n "__fish_postar_needs_command" -l polling-delay -d 'The polling delay when using the polling method for inboxes' -r
complete -c postar -n "__fish_postar_needs_command" -l check -d 'Check whether the configuration is valid'
complete -c postar -n "__fish_postar_needs_command" -l dry-run -d 'Perform a dry run on the most recent 10 messages'
complete -c postar -n "__fish_postar_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c postar -n "__fish_postar_needs_command" -s V -l version -d 'Print version'
complete -c postar -n "__fish_postar_needs_command" -f -a "completions" -d 'Outputs shell completions to stdout'
complete -c postar -n "__fish_postar_needs_command" -f -a "init" -d 'Intializes the configuration files'
complete -c postar -n "__fish_postar_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c postar -n "__fish_postar_using_subcommand completions" -s h -l help -d 'Print help'
complete -c postar -n "__fish_postar_using_subcommand init" -l custom-path -d 'Custom output path for the config file' -r -F
complete -c postar -n "__fish_postar_using_subcommand init" -l write-example-rules -d 'Whether to write a sample rules.ptar file to the default path'
complete -c postar -n "__fish_postar_using_subcommand init" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c postar -n "__fish_postar_using_subcommand help; and not __fish_seen_subcommand_from completions init help" -f -a "completions" -d 'Outputs shell completions to stdout'
complete -c postar -n "__fish_postar_using_subcommand help; and not __fish_seen_subcommand_from completions init help" -f -a "init" -d 'Intializes the configuration files'
complete -c postar -n "__fish_postar_using_subcommand help; and not __fish_seen_subcommand_from completions init help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
