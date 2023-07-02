CL65	= cl65
LD65	= ld65

#-------------------------------------------------------------------------------
CSOURCES =	sample2.c

ASMSOURCES =	startup.asm

OBJECTS	=	$(CSOURCES:.c=.o) $(ASMSOURCES:.asm=.o)

LIBRARIES =	
#-------------------------------------------------------------------------------
all :	$(OBJECTS) $(LIBRARIES)
	LD65 -o sample2.nes --config sample2.cfg --obj $(OBJECTS) --lib nes.lib

.SUFFIXES : .asm .o

.c.o :
	CL65 -t none -o $*.o -c -O $*.c

.asm.o :
	CL65 -t none -o $*.o -c $*.asm

clean :
	del *.nes
	del *.o
