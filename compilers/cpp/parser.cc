#include<ctype.h>
#include"parser.h"
using namespace std;

bool isparen(char c) {
  return c == '(' || c == ')';
}

bool isquote(char c) {
  return c == '\'';
}

void tokenize(istream& in, vector<string>& tokens) {
  string s;
  while (getline(in, s)) {
    int i = 0;
    while (i < s.length()) {
      if (isblank(s[i])) {
        i++;
      } else if (isparen(s[i]) || isquote(s[i])) {
        string buf = "";
        buf += s[i++];
        tokens.push_back(buf);
      } else if (s[i] == '"') {
        string buf = "\"";
        char c;
        i++;
        do {
          c = s[i++];
          buf += c;
          if (c == '\\') {
            buf += s[i++];
          }
        } while (c != '"');
        tokens.push_back(buf);
      } else {
        string buf = "";
        while (!isspace(s[i]) && !isparen(s[i])) {
          buf += s[i++];
        }
        tokens.push_back(buf);
      }
    }
  }
}

void printSpace(int n) {
  while (n--) {
    cout << " ";
  }
}

void List::display(int indent) {
  cout << "(";
  for (int i = 0; i < exprs.size(); i++) {
    if (i) {
      cout << endl;
      printSpace(indent + 2);
    }
    exprs[i]->display(indent + 2);
  }
  cout << ")";
}

Expr* parse(const vector<string>& tokens, int& i) {
  if (i >= tokens.size()) {
    return nullptr;
  }
  if (tokens[i] == "(") {
    i++;
    List* res = new List();
    while (tokens[i] != ")") {
      res->exprs.push_back(parse(tokens, i));
    }
    i++;
    return res;
  } else if (tokens[i] == "'") {
    i++;
    List* res = new List();
    res->exprs.push_back(new Symbol("quote"));
    res->exprs.push_back(parse(tokens, i));
    return res;
  } else if (tokens[i][0] == '"') {
    return new String(tokens[i++]);
  } else if (tokens[i][0] == '#') {
    return new Bool(tokens[i++]);
  } else if (isdigit(tokens[i][0])) {
    return new Integer(tokens[i++]);
  }
  return new Symbol(tokens[i++]);
}

vector<Expr*> parse(istream& in) {
  vector<string> tokens;
  tokenize(in, tokens);
  int i = 0;
  vector<Expr*> res;
  while (i < tokens.size()) {
     res.push_back(parse(tokens, i));
  }
  return res;
}
