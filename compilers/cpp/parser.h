#ifndef __PARSER_H__
#define __PARSER_H__

#include<string>
#include<vector>
#include<iostream>
using namespace std;

struct Expr {

  virtual void display(int indent = 0) = 0;
};

struct Leaf : public Expr {
  
  string s;

  Leaf(string s): s(s) {}

  virtual void display(int indent) {
    cout<<s;
  }
};

struct Bool : public Leaf {

  Bool(string s): Leaf(s) {}
};

struct Integer : public Leaf {

  Integer(string s): Leaf(s) {}
};

struct String : public Leaf {

  String(string s): Leaf(s) {}
};

struct Symbol : public Leaf {

  Symbol(string s): Leaf(s) {}
};

struct List : public Expr {
  
  vector<Expr*> exprs;

  virtual void display(int indent = 0);
};

vector<Expr*> parse(istream& in);

#endif
