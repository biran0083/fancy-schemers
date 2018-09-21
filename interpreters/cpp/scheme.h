#ifndef __SCHEME_H__
#define __SCHEME_H__

#include <string>
#include <map>
using namespace std;

struct Environment;

struct Expression {
  virtual Expression* eval(Environment*) = 0;
  virtual string toString() = 0;
};

struct Environment {
  Environment* parent;
  map<string, Expression*> m;

  Environment() : parent(0) {}

  bool has(string s);

  void put(string s, Expression* e);
  
  Expression* get(string s);

  void show();
};

Expression* parse(string s);

#endif
