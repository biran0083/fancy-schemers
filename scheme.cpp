#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <assert.h>
#include <map>
#include <stack>
#include <list>
#include <vector>
#include <iostream>
#include <sstream>
#define ISA(v,type) (bool)(dynamic_cast<type>(v))
using namespace std;

struct Environment;
struct Expression {
  virtual Expression* eval(Environment*)=0;
  virtual string toString()=0;
};
struct Null:Expression{
  virtual string toString(){
    return "Null";
  }
  virtual Expression* eval(Environment* e){return this;}
};
struct Environment{
  Environment* parent;
  map<string,Expression*> m;
  Environment():parent(0){}
  bool has(string s){
    bool res = m.find(s)!=m.end() || (parent && parent->has(s));
    return res;
  }
  void put(string s,Expression* e){
    m[s]=e;
  }
  Expression* get(string s){
    return m.find(s)==m.end() ? parent->get(s) : m[s];
  }
  void show(){
    for(map<string,Expression*>::iterator it=m.begin();
        it!=m.end();it++)
      cout<<it->first<<" "<<it->second->toString()<<endl;
    if(parent)parent->show();
  }
};
struct ExpSequence: public Expression{
  vector<Expression*> es;
  virtual Expression* eval(Environment* e){
    assert(es.size()>0);
    Expression* res = 0;
    for(int i=0;i<es.size();i++)
      res = es[i]->eval(e);
    return res;
  }
  virtual string toString(){
    string res="";
    for(int i=0;i<es.size();i++)
      res+=es[i]->toString();
    return res;
  }
};
struct BoolValue: public Expression{
  bool v;
  BoolValue(bool v):v(v){}
  virtual Expression* eval(Environment* e){return this;}
  virtual string toString(){
    return v?"#t":"#f";
  }
};
struct IntValue:public Expression{
  int v;
  IntValue(int v):v(v){}
  virtual Expression* eval(Environment* e){return this;}
  virtual string toString(){
    stringstream str;
    str<<v;
    return str.str();
  }
};
struct Label:public Expression{
  string s;
  Label(string s):s(s){}
  virtual Expression* eval(Environment* e){
    return e->has(s) ? e->get(s) : this;
  }
  virtual string toString(){
    return s;
  }
};
struct Lambda:public Expression{
  vector<string> args;
  Expression* body;
  Environment* e;
  Lambda():e(0){}
  Lambda(Environment* e):e(e){}
  virtual Expression* eval(Environment* e){
    Lambda* res= new Lambda(*this); 
    res->e=e;
    return res;
  }
  virtual string toString(){
    string s="(lambda (";
    for(int i=0;i<args.size();i++){
      if(i)s+=" ";
      s+=args[i];
    }
    s+=") "+body->toString()+")";
    return s;
  }
};
struct Pair:public Expression{
  Expression *a,*b;
  Pair(Expression*a,Expression*b):a(a),b(b){}
  virtual Expression* eval(Environment* e){return this;}
  virtual string toString(){
    string res = "("+a->toString();
    Expression* t=b;
    while(ISA(t,Pair*)){
      Pair* p=(Pair*) t;
      res += " " + p->a->toString();
      t = p->b;
    }
    if(ISA(t,Null*)){
      res+=")";
    }else{
      res+=" . "+t->toString()+")";
    }
    return res;
  }
};
struct Application:Expression{
  Expression* op;
  vector<Expression*> args;
  virtual string toString(){
    string s="("+op->toString();
    for(int i=0;i<args.size();i++)
      s+=" "+args[i]->toString();
    s+=")";
    return s;
  }
  virtual Expression* eval(Environment* e){
    Expression* opVal = op->eval(e);
    vector<Expression*> vals(args.size());
    for(int i=0;i<args.size();i++)
      vals[i]=args[i]->eval(e);
    Lambda* f=0;
    if(ISA(opVal,Label*)){
      string s = ((Label*)opVal)->s;
      if(e->has(s)) f = (Lambda*)(e->get(s));
      else if(s=="cons") return new Pair(vals[0],vals[1]);
      else if(s=="car") return ((Pair*)vals[0])->a;
      else if(s=="cdr") return ((Pair*)vals[0])->b;
      else if(s=="list"){
        int i=vals.size()-1;
        if(i<0)return new Null();
        Pair* res=new Pair(vals[i--],new Null());
        while(i>=0)
          res=new Pair(vals[i--],res);
        return res;
      } else if(s=="+")return new IntValue(((IntValue*)vals[0])->v + ((IntValue*)vals[1])->v);
      else if(s=="-")return new IntValue(((IntValue*)vals[0])->v - ((IntValue*)vals[1])->v);
      else if(s=="*")return new IntValue(((IntValue*)vals[0])->v * ((IntValue*)vals[1])->v);
      else if(s=="/")return new IntValue(((IntValue*)vals[0])->v / ((IntValue*)vals[1])->v);
      else if(s=="mod")return new IntValue(((IntValue*)vals[0])->v % ((IntValue*)vals[1])->v);
      else if(s==">")return new BoolValue(((IntValue*)vals[0])->v > ((IntValue*)vals[1])->v);
      else if(s=="<")return new BoolValue(((IntValue*)vals[0])->v < ((IntValue*)vals[1])->v);
      else if(s=="and")return new BoolValue(((BoolValue*)vals[0])->v && ((BoolValue*)vals[1])->v);
      else if(s=="or")return new BoolValue(((BoolValue*)vals[0])->v || ((BoolValue*)vals[1])->v);
      else if(s=="not")return new BoolValue(!((BoolValue*)vals[0])->v);
      else if(s=="null?")return new BoolValue(ISA(vals[0],Null*));
      else if(s=="eq?" || s=="="){
        if(ISA(vals[0],BoolValue*) && ISA(vals[1],BoolValue*))
          return new BoolValue(((BoolValue*)vals[0])->v == ((BoolValue*)vals[1])->v);
        else if(ISA(vals[0],IntValue*) && ISA(vals[1],IntValue*))
          return new BoolValue(((IntValue*)vals[0])->v == ((IntValue*)vals[1])->v);
        else if(ISA(vals[0],Null*) && ISA(vals[1],Null*))
          return new BoolValue(true);
        else return new BoolValue(false);
      }
      else{
        cout<<"unknown label: "<<s<<endl;
        assert(false);
      }
    }else if(ISA(opVal,Lambda*)) f = (Lambda*)opVal;
    Environment* e2 = new Environment();
    e2->parent = f->e;
    for(int i=0;i < f->args.size();i++)
      e2->m[f->args[i]] = vals[i];
    return f->body->eval(e2);
  }
};
struct If:Expression{
  Expression *condition, *thenPart, *elsePart;
  virtual string toString(){
    return "(if "+condition->toString()+" "+thenPart->toString()+" "+elsePart->toString()+")";
  }
  virtual Expression* eval(Environment* e){
    BoolValue* t = (BoolValue*)(condition->eval(e));
    if(t->v)return thenPart->eval(e);
    else return elsePart->eval(e);
  }
};
struct Define:Expression{
  Label *label;
  Expression* exp;
  virtual string toString(){
    return "(define "+label->s+" "+exp->toString()+")"; 
  }
  virtual Expression* eval(Environment* e){
    e->put(label->s,exp->eval(e));
    return new Null();
  }
};
void trim(string& s){
  int i=0;
  while(i<s.size() && s[i]==' ')
    i++;
  s=s.substr(i);
}
string parseToken(string &s){
  trim(s);
  if(s=="")return "";
  int i=0;
  if(s[i]=='(' || s[i]==')'){
    i++;
  }else{
    while(i<s.size() && s[i]!=' ' && s[i]!=')' && s[i]!='(')
      i++;
  }
  string res=s.substr(0,i);
  s=s.substr(i);
  return res;
}
bool isInteger(string s){
  int i=0;
  if(s[0]=='-')i++;
  if(!s[i])return false;
  for(;s[i];i++)
    if(s[i]<'0' || s[i]>'9')return false;
  return true;
}
Expression* parse(list<string>& tokens){
  if(tokens.front()=="("){
    tokens.pop_front();
    if(tokens.front()==")"){
      tokens.pop_front();
      return new Null();
    }if(tokens.front()=="lambda"){
      tokens.pop_front();
      Lambda* res=new Lambda();
      assert(tokens.front()=="(");
      tokens.pop_front();
      while(tokens.front()!=")"){
        res->args.push_back(tokens.front());
        tokens.pop_front();
      }
      tokens.pop_front();
      ExpSequence* exps=new ExpSequence();
      do{
        exps->es.push_back(parse(tokens));
      }while(tokens.front()!=")");
      tokens.pop_front();
      res->body = exps;
      return res;
    }else if(tokens.front()=="if"){
      tokens.pop_front();
      If *res = new If();
      res->condition = parse(tokens);
      res->thenPart = parse(tokens);
      res->elsePart = parse(tokens);
      tokens.pop_front();
      return res;
    }else if(tokens.front()=="define"){
      tokens.pop_front();
      Define *res = new Define();
      if(tokens.front()=="("){
        tokens.pop_front();
        res->label = (Label*)(parse(tokens));
        Lambda* f=new Lambda();
        while(tokens.front()!=")"){
          f->args.push_back(tokens.front());
          tokens.pop_front();
        }
        tokens.pop_front();
        ExpSequence* exps=new ExpSequence();
        do{
          exps->es.push_back(parse(tokens));
        }while(tokens.front()!=")");
        tokens.pop_front();
        f->body = exps;
        res->exp=f;
      }else{
        res->label = (Label*)(parse(tokens));
        res->exp = parse(tokens);
        tokens.pop_front();
      }
      return res;
    }else if(tokens.front()=="let"){
      tokens.pop_front();
      assert(tokens.front()=="(");
      tokens.pop_front();
      Application *res=new Application();
      Lambda* f=new Lambda();
      res->op=f;
      while(tokens.front()!=")"){
        assert(tokens.front()=="(");
        tokens.pop_front();
        f->args.push_back(((Label*)parse(tokens))->s);
        res->args.push_back(parse(tokens));
        tokens.pop_front();
      }
      tokens.pop_front();
      f->body=parse(tokens);
      tokens.pop_front();
      return res;
    }else{
      Application* res=new Application();
      res->op = parse(tokens);
      while(tokens.front()!=")"){
        res->args.push_back(parse(tokens));
      }
      tokens.pop_front();
      return res;
    }
  }else{
    string s=tokens.front();
    tokens.pop_front();
    if(s[0]=='#'){
      return new BoolValue(s[1]=='t');
    }else if(isInteger(s)){
      return new IntValue(atoi(s.c_str()));
    }
    return new Label(s);
  }
}
Expression* parse(string s){
  list<string> tokens;
  while(1){
    string t=parseToken(s);
    if(t=="")break;
    tokens.push_back(t);
  }
  ExpSequence* res=new ExpSequence();
  while(tokens.size()>0)
    res->es.push_back(parse(tokens));
  return res;
}
int main(int argc, const char *argv[])
{
  string code;
  Environment* env = new Environment();
  while(getline(cin,code)){
    Expression * e=parse(code);
    cout<<"exp: "<<e->toString()<<endl;
    cout<<"res: "<<e->eval(env)->toString()<<endl;
  }
  return 0;
}
