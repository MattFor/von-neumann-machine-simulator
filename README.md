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

| Key      | Action             |
| -------- | ------------------ |
| `F5`     | Run / pause        |
| `F10`    | Step               |
| `Ctrl+R` | Reset the CPU      |
| `Ctrl+O` | Open a program     |
| `Ctrl+S` | Save the program   |

- **Memory**: each cell is a 16-bit word, the high byte is the instruction code and
  the low byte is the operand. Edit a cell through its instruction, its operand or
  its whole value (`Val`). The highlighted row is the program counter and the view
  follows it.
- **Breakpoints**: click an address to toggle one. Run stops when the program
  counter reaches it; press Run again to carry on.
- **Registers**: the ones the last step changed are highlighted.
- **Console**: `OUT` prints here. When the program stops on `IN`, type one or more
  numbers and press Enter; a running program carries on by itself.
- **Instruction Set**: rename instructions, change their codes or add your own
  aliases, and save the set as JSON.

Speed, visible panels, theme and window size are remembered between launches.

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
| 0B   | INC   | `mem[operand] += 1`                         |
| 0C   | DEC   | `mem[operand] -= 1`                         |
| 0D   | NULL  | `mem[operand] = 0`                          |
| 0E   | TST   | Skip the next instruction if `mem[operand] == 0` |
| FF   | HALT  | Stop                                        |

The accumulator and the data bus are kept between 0 and 19999.

## Program files

Programs are JSON, see [`assets/countdown.json`](assets/countdown.json):

```json
{ "name": "countdown", "entry_point": 0, "memory": [266, 2560, 3082] }
```

`memory` lists the words from address 0. Saved programs also carry the instruction
set they were written with, under `instruction_set`, and opening one switches to
that set. Files without it keep the set that is currently loaded.
