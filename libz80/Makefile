SOURCES = z80.c
FLAGS_SO = -fPIC -Wall -ansi -O3
FLAGS = -fPIC -Wall -ansi -O3 -c

force: clean all

all: libz80.so

libz80.so: z80.h $(OBJS)
	cd codegen && make opcodes
	gcc $(FLAGS_SO) -shared -o libz80.so $(SOURCES)

libz80.a: z80.h $(OBJS)
	cd codegen && make opcodes
	gcc $(FLAGS) $(SOURCES) 
	ar rcs libz80.a z80.o

install:
	install -m 666 libz80.so /usr/lib
	install -m 666 z80.h /usr/include

clean:
	rm -f *.a *.o *.so core
	cd codegen && make clean

realclean: clean
	rm -rf doc

doc:	*.h *.c
	doxygen

