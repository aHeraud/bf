# bf

An interpreter + recompiler for the [brainfuck esoteric programming language](https://esolangs.org/wiki/brainfuck). Uses 8-bit wrapping memory cells, and a 30,000 byte memory array (no negative indices please!).

## Usage

To run a program, just call `bf <sourcefile>`. This will run the program by translating it into x86_64 machine code and running it as native code. To run a program using the interpreter instead, use `bf -i <sourcefile>`.

## Example

![](images/mandlebrot.png)

## Benchmarks

### mandlebrot

Interpreter: 6.211s

Recompiler: 1.054s

![](images/mandlebrot_benchmark.png)
