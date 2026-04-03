# Rescue Me Claude - Root bashrc

# Standard PATH
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

# Node.js global bin
export PATH="$PATH:/usr/lib/node_modules/.bin"

# Nice prompt
PS1='\[\e[1;31m\][rescue-claude]\[\e[0m\] \[\e[1;34m\]\w\[\e[0m\] # '

# Aliases
alias ll='ls -lah --color=auto'
alias la='ls -A --color=auto'
alias l='ls -CF --color=auto'
alias ..='cd ..'
alias grep='grep --color=auto'

# Show MOTD on login
if [ -z "$DISPLAY" ] && [ "$(tty)" = "/dev/tty1" ]; then
    cat /etc/motd
fi
