#include <iostream>
#include "scheme.h"
int main(int argc, const char* argv[]) {
  string code;
  Environment* env = new Environment();
  while (getline(cin, code)) {
    Expression* e = parse(code);
    cout << "exp: " << e->toString() << endl;
    cout << "res: " << e->eval(env)->toString() << endl;
  }
  return 0;
}
