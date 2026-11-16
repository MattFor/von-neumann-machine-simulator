# Von Neumann Machine Simulator

[![CI](https://github.com/MattFor/von-neumann-machine-simulator/actions/workflows/ci.yml/badge.svg)](https://github.com/MattFor/von-neumann-machine-simulator/actions/workflows/ci.yml)

A desktop simulator of a simple accumulator machine: memory, registers, a CPU and the data bus between them, with a console for input and output.

## Usage

Pick a program from **Example**, then **Run** or **Step**. There is one for every
machine, and it loads on its machine. The **Guide** tab explains everything in the
app, and every button has a tooltip.

The app starts in a simple view: memory, registers and the data bus. **Advanced**
(top right) adds the control unit, the buttons to drive the machine by hand and the
view switches. **Machine** next to it picks the machine to simulate.

| Key      | Action                            |
| -------- | --------------------------------- |
| `F5`     | Run / pause                       |
| `F10`    | Step                              |
| `F9`     | Back: undo the last step          |
| `F11`    | Micro step (control unit visible) |
| `Ctrl+R` | Reset the CPU                     |
| `Ctrl+O` | Open a program                    |
| `Ctrl+S` | Save the program                  |

The keys work on the Machine tab.

- **Memory**: each cell is one word, an instruction code and then an operand. Edit a
  cell through its instruction, its operand or the number it holds (`Val`, negative
  numbers included). The instruction column shows what the cell runs as. The orange
  row is the program counter and the view follows it, the blue row is the cell the
  address bus points at.
- **Breakpoints**: click an address to toggle one. Run stops when the program
  counter reaches it; press Run again to carry on.
- **Back**: undoes the last step, micro step or micro step button, up to 4096 of them.
  Memory, registers, flags, console output, the input queue, the program counter and
  whether the machine halted or waits for input go back to how they were. Loading a
  program, Reset, Clear and editing a memory cell by hand forget the steps before, so
  Back has nothing to undo after them. Each machine keeps its own steps, so switching
  machines loses none.
- **Registers**: the ones the last step changed are highlighted. A machine with flags,
  like SAP-1, also shows its carry (`CF`) and zero (`ZF`) flags.
- **Console**: `OUT` prints here. When the program stops on `IN`, type one or more
  numbers and press Enter; a running program carries on by itself. `0x` and `0b`
  values are words in the machine's encoding, as with `--input`.
- **Control unit** (advanced view): shows the micro steps of the
  current instruction. **Micro Step** does one of them at a time. Every micro step
  the loaded set uses is also a button, so the machine can be driven by hand, and
  **Record** turns the buttons you press into a new instruction.
- **Instruction Set**: rename instructions, change their codes or add your own
  aliases, and save the set as JSON. An instruction set to `Custom` has its micro
  steps edited right there. An instruction can have a fixed operand, like LMC's
  `INP` (901) and `OUT` (902), and a mask, like MARIE's `Skipcond`.
- **Format**, on the Instruction Set tab: the radix, the digits of code and operand,
  how many codes and cells there are, the accumulator range, how negative numbers
  are stored, what an unknown code or the end of memory does, whether Run stops on an
  instruction that leaves the program counter where it was, and whether plus and
  minus set flags, like SAP-1's. A format the
  machine can't hold is not applied, and the tab says why.

### Machines

The **Machine** picker has seven presets. Each keeps its own program, so switching
loses nothing.

| Machine             | Words                                                       |
| ------------------- | ----------------------------------------------------------- |
| Standard            | Hex: a byte of code, a byte of operand. 256 cells           |
| Johnny              | Decimal: two digits of code, three of operand. 1000 cells   |
| Bonsai              | Johnny's words with its minimal set: INC, DEC, JMP, TST, HLT |
| Little Man Computer | Three decimal digits, 100 mailboxes, values -999 to 999     |
| MARIE               | 16-bit hex: one digit of code, three of operand. 4096 cells |
| CARDIAC             | A sign and three decimal digits, 100 cells, a four-digit accumulator |
| SAP-1               | A byte: one hex digit of code, one of address. 16 cells, an 8-bit A register |

CARDIAC is Bell Labs' cardboard computer. Its cell 00 always holds `001`, the
loader, and can't be edited: run from 00, it reads the next input into 01 and
carries on there. `JMP` leaves a jump back (`8xx`) in cell 99, `SFT xy` shifts the
accumulator `x` digits left, then `y` right, and `STO` keeps the accumulator's low
three digits.

SAP-1 is Ben Eater's breadboard computer with the instructions from the end of his
series: `NOP`, `LDA`, `ADD`, `SUB`, `STA`, `LDI n` (loads its 4-bit operand), `JMP`,
`JC`, `JZ`, `OUT` and `HLT`. Its A register goes round from 255 to 0 and back.
`ADD` and `SUB` set the carry flag (on `SUB`, when nothing was borrowed) and the zero
flag, and `JC` and `JZ` jump on them; nothing else changes the flags. The registers
show them as `CF` and `ZF`.

Johnny and Bonsai work like the [Johnny simulator](https://github.com/Laubersheini/johnny).
**Open Program** reads Johnny's `.ram` files and **Open Set** its `.mc` microcode
files. As in Johnny, they load into any machine of Johnny's words, like Bonsai or a
set from a `.mc` file; from any other machine they switch to Johnny, and the machine
left is kept as it was. Saving a program under a name ending in `.ram` writes a Johnny
file. It holds only the words, no set or entry point, so any program in Johnny's words
that starts at 0 is saved that way, Bonsai's too.

Once the set or its format matches none of the presets, the picker says **Custom**.
Picking the machine again gives it its own set back and keeps the memory.

Speed, visible panels, the view, the machine, theme and window size are remembered
between launches.

### Instructions

The Standard machine's set:

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

The accumulator is kept between 0 and 19999. An operand reaches 256 cells (1000 on
Johnny); after the last one the program counter goes back to 0, on Johnny it stays on
999 like in Johnny. The Guide tab describes the machine that is loaded.

Every instruction is a list of micro steps (`ins->ab`, `ram->db`, `db->acc`, ...) that
runs after the fetch (`pc->ab`, `ram->db`, `db->ins`, `ins->mc`). The Guide tab and the
Instruction Set tab list them all.

### Program files

Programs are JSON, see [`assets/countdown.json`](assets/countdown.json):

```json
{ "name": "countdown", "entry_point": 0, "memory": [266, 2560, 3082] }
```

`memory` lists the words from address 0. Saved programs also carry the instruction
set they were written with, under `instruction_set`, and opening one switches to
that set. Files without it keep the set that is currently loaded.

Files ending in `.asm` are programs as text, in the mnemonics of the loaded set:

```asm
start:  LOAD count   ; comments start with ;, # or MARIE's /
        OUT
        DEC count
        JZ end
        JMP start
end:    HALT
count:  DAT 5
```

- Mnemonics can be in any case; two that differ only in case can't be used.
  Instructions with a fixed operand, like LMC's `INP` and `OUT`, take none; MARIE's
  `Skipcond` takes its condition, as in `Skipcond 400`.
- Numbers are in the set's radix unless `0x`, `0b` or `0d` says otherwise, and a
  label can stand in for one. Where B or D is a digit, as in hex, `0B` and `0D` are
  just numbers, so MARIE's `HEX 0D05` is `D05`. A label is letters, digits and `_`, not
  starting with a digit. It ends in `:` or MARIE's `,`, or is LMC's first word that is
  not a mnemonic (a directive's name counts as a label there).
- `DAT n` (`DAT` alone is 0) writes a number, as do MARIE's `DEC n` and `HEX n`. On
  MARIE, `DEC -1` is stored as `FFFF`. An instruction of the same name wins over
  any directive, like Johnny's `DEC`.
- `ORG n` carries on at address `n`; going back over a filled cell is an error.
  CARDIAC's fixed cell 00 counts as filled, so its programs start with an `ORG`.
- `END` stops reading, like in MARIE. `END n` or `END label` also says where the
  program starts; without it, it starts at its first instruction or number.

Saving under `.asm` writes this form: words that are no instruction come out as
`DAT` (MARIE's negative ones as `DEC`), and a start other than 0 as `END n`.

Files ending in `.mas` are MARIE's own assembly. They are always read in MARIE's
mnemonics, whatever machine is loaded, and opening one switches to MARIE. Saving
a MARIE program under `.mas` writes one.

### Command line

`run` runs a program without the window and prints its output, one value per line:

```sh
von-neumann-machine-simulator run tests/programs/lmc-add.asm --machine lmc --input 12,30
```

- `--machine standard|johnny|bonsai|lmc|marie|cardiac|sap1` picks the machine. Without it, a file
  runs on the set it carries, `.ram` on Johnny and JSON on Standard; `.asm` needs it.
  A `.ram` file runs on `johnny` or `bonsai`, a `.mas` file only on `marie`, and every
  file is checked against the machine it runs on, as **Open Program** does.
- `--input 1,2,-3` is what `IN` reads. `0x` and `0b` values are words in the machine's
  encoding, so `0xFFFF` is -1 on MARIE.
- `--max-steps N` stops after `N` instructions (100000 by default).
- `--trace` prints every instruction to stderr: address, word, mnemonic and the
  accumulator after it.

The exit code says how it ended: 0 halted, 1 bad arguments or file, 2 step limit
reached, 3 waiting for input with none left, 141 the reader went away (`| head`), as with
other tools. A run also says which on stderr. `--help` lists all of it. Without
arguments the window opens.
[`tests/programs`](tests/programs) has textbook programs for every machine.

`convert` loads a program and saves it in the format of the output file's extension,
`.json`, `.ram`, `.asm` or `.mas`:

```sh
von-neumann-machine-simulator convert tests/programs/marie-add.asm add.mas --machine marie
```

`--machine` works as for `run`. A program the output format cannot hold, like a Standard one
as `.ram`, is refused with exit code 1; a conversion that works exits with 0.

## Building

```sh
cargo run --release
cargo run --release --features debug-mode   # adds the Debug tab and panel outlines
cargo test
```

## Contributing

Created By MattFor and Krisunio
