from mp import *

REMOTE_IP = os.getenv("REMOTE_IP")
REMOTE_PORT = os.getenv("REMOTE_PORT")
TICKET = os.getenv("TICKET", "ticket")

from paradigmctf.eth_challenge import *

bytecode = compile(True)

p = remote(REMOTE_IP, int(REMOTE_PORT))

p >> "action?" << "1\n"

p >> "ticket please:" << TICKET << "\n"

p >> "runtime bytecode:" << bytes.hex(bytecode) << '\n'

p >> "you didn't factor the number. "

p >> "\n"

(a, b) = p.scan("%d * %d")

print(bytes.fromhex(hex(a << 256 | b)[2:]).split(b'\x00')[0].decode('utf8'))