#!/bin/bash

#
# fake-solver.sh: a fake argumentation solver
# Copyright (C) 2020  Artois University and CNRS
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http:#www.gnu.org/licenses/>.
#
# Contributors:
#   *   CRIL - initial API and implementation
#
# author: 
# Emmanuel Lonca <lonca@cril.fr>

#
# This scripts creates a fake argumentation solver which outputs the content
# sent to it using the netcat command.
#
# Just execute it and follow the instructions logged on the console.
#


TCP_PORT=8081

function log {
    echo "[FAKESOLVER] $*" > /dev/stderr
}

function log_inline {
    echo -n "[FAKESOLVER] $*" > /dev/stderr
}

function read_line {
    log_inline "OUTPUT: " > /dev/stderr
    read r
    echo "$r"
}

log "arguments: $*"
log
log "to send a fake solver output, use:"
log "echo FAKE_SOLVER_OUTPUT | nc -tN localhost $TCP_PORT"
log
log "to kill the fake solver output, use:"
log "kill $$"
log
while [ true ]
do
    log "waiting for fake solver output"
    solver_output=`nc -tl $TCP_PORT`
    log_inline "SENT: "
    echo "$solver_output"
    sleep 1
    read solver_input
    log "GOT $solver_input"
    log
done
