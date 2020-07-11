load Main.vm;

set RAM[8000] 255;
// set RAM[8000] 42;
repeat 1000000 {
  vmstep;
}
