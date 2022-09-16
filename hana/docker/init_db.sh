#!/bin/bash
set -euxo pipefail

rm -f hana-ctf.db

sqlite3 hana-ctf.db \
"create table players (
    id integer primary key,
    pubkey text unique not null,
    last_signature text not null default ''
)" \
"create table player_programs (
    id integer primary key,
    player_id integer not null,
    pubkey text unique not null,
    foreign key(player_id) references players(id)
)" \
"create table flags (
    id integer primary key,
    challenge integer unique not null,
    flag text not null
)" \
"insert into flags (challenge, flag) values (1, 'unsafe_voucher_vouched_vouchsafely')" \
"insert into flags (challenge, flag) values (2, 'perfectly_precise_unfortunately_inaccurate')" \
"insert into flags (challenge, flag) values (3, 'if_you_copied_code_from_the_adobe_repo_youre_weak')"
