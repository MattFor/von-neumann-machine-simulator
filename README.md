# Von Neumann Machine Simulator

A desktop simulator of a simple accumulator machine: memory, registers, a CPU
and the data bus between them, with a console for input and output.

Created By MattFor and Krisunio

## Run

```sh
cargo run --release
cargo run --release --features debug-mode   # adds the Debug tab and panel outlines
cargo test
```

## Use

Click **Example** to load a small countdown program, then **Run** or **Step**.
The **Guide** tab explains everything in the app, and every button has a tooltip.

The app starts in a simple view: memory, registers and the data bus. **Advanced**
(top right) adds the control unit, the buttons to drive the machine by hand and the
view switches. **Johnny mode** next to it makes the machine work like the
[Johnny simulator](https://github.com/Laubersheini/johnny).

| Key      | Action             |
| -------- | ------------------ |
| `F5`     | Run / pause        |
| `F10`    | Step               |
| `F11`    | Micro step         |
| `Ctrl+R` | Reset the CPU      |
| `Ctrl+O` | Open a program     |
| `Ctrl+S` | Save the program   |

- **Memory**: each cell is a 16-bit word, the high byte is the instruction code and
  the low byte is the operand. Edit a cell through its instruction, its operand or
  its whole value (`Val`). The orange row is the program counter and the view
  follows it, the blue row is the cell the address bus points at.
- **Breakpoints**: click an address to toggle one. Run stops when the program
  counter reaches it; press Run again to carry on.
- **Registers**: the ones the last step changed are highlighted.
- **Console**: `OUT` prints here. When the program stops on `IN`, type one or more
  numbers and press Enter; a running program carries on by itself.
- **Control unit** (advanced view): shows the micro steps of the
  current instruction. **Micro Step** does one of them at a time. Every micro step
  is also a button, so the machine can be driven by hand, and **Record** turns the
  buttons you press into a new instruction.
- **Instruction Set**: rename instructions, change their codes or add your own
  aliases, and save the set as JSON. An instruction set to `Custom` has its micro
  steps edited right there.
- **Johnny mode**: decimal words (two digits of instruction code, three of operand),
  1000 cells and Johnny's instructions. Each mode keeps its own program, so switching
  loses nothing. **Open Program** reads Johnny's `.ram` files and **Open Set** its
  `.mc` microcode files, both switch to Johnny mode by themselves. Saving a program
  under a name ending in `.ram` writes a Johnny file. **Bonsai** on the Instruction
  Set tab loads Johnny's minimal set.

Speed, visible panels, the view, the mode, theme and window size are remembered
between launches.

## Instructions

| Code | Name  | Effect                                      |
| ---- | ----- | ------------------------------------------- |
| 00   | NOP   | Nothing                                     |
| 01   | LOAD  | `ACC = mem[operand]`                        |
| 02   | STORE | `mem[operand] = ACC`                        |
| 03   | ADD   | `ACC += mem[operand]`                       |
| 04   | SUB   | `ACC -= mem[operand]`                       |
| 05   | MUL   | `ACC *= operand`                            |
| 06   | DIV   | `ACC /= operand`, halts on zero             |
| 07   | JMP   | `PC = operand`                              |
| 08   | JZ    | `PC = operand` if `ACC == 0`                |
| 09   | IN    | `ACC = next input value`                    |
| 0A   | OUT   | Print `ACC`                                 |
| 0B   | INC   | `mem[operand] += 1`, through `ACC`          |
| 0C   | DEC   | `mem[operand] -= 1`, through `ACC`          |
| 0D   | NULL  | `mem[operand] = 0`, `ACC = 0`               |
| 0E   | TST   | `ACC = mem[operand]`, skip the next instruction if it is 0 |
| FF   | HALT  | Stop                                        |

The accumulator is kept between 0 and 19999.

Every instruction is a list of micro steps (`ins->ab`, `ram->db`, `db->acc`, ...) that
runs after the fetch (`pc->ab`, `ram->db`, `db->ins`, `ins->mc`). The Guide tab and the
Instruction Set tab list them all.

## Program files

Programs are JSON, see [`assets/countdown.json`](assets/countdown.json):

```json
{ "name": "countdown", "entry_point": 0, "memory": [266, 2560, 3082] }
```

`memory` lists the words from address 0. Saved programs also carry the instruction
set they were written with, under `instruction_set`, and opening one switches to
that set. Files without it keep the set that is currently loaded.
