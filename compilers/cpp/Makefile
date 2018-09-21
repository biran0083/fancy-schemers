%.o: %.cc %.h
	g++ -c $< --std=c++14 -o $@

parser: parser.o
	g++ parser.o parser_main.cc --std=c++14 -o parser

all: parser
