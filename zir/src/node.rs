#[derive(Debug, PartialEq)]
pub enum AbstractType {
    Integer,
    Boolean,
    String,
    Void
}

#[derive(Debug)]
pub enum Node {
	Int(i32),
	Add {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	Subtract {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	Multiply {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	Divide {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	Equal {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	NotEqual {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	LessThanOrEqual {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	GreaterThanOrEqual {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	LessThan {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	GreaterThan {
		lhs: Box<Node>,
		rhs: Box<Node>
	},

	Bool(bool),
	BoolEqual {
		lhs: Box<Node>,
		rhs: Box<Node>
	},
	BoolNotEqual {
		lhs: Box<Node>,
		rhs: Box<Node>
	},

	StringLiteral(String),

	Set {
		name: String,
		var_type: AbstractType,
		value: Box<Node>
	},
	Get {
		name: String
	},

	Function {
		name: String,
		args: Vec<Node>
	}
}

#[derive(Debug)]
pub enum Statement {
	Node(Node),
	Return(Node),
	If {
		condition: Node,
		branch: Vec<Statement>,
		else_branch: Vec<Statement>
	},
	While {
		condition: Node,
		loop_statements: Vec<Statement>
	},
	InfiniteLoop(Vec<Statement>)
}

#[derive(Debug)]
pub struct Definition {
	pub name: String,
	pub data_type: AbstractType
}

#[derive(Debug)]
pub struct FunctionInfo {
	pub body: Vec<Statement>,
	pub args: Vec<Definition>,
	pub return_type: AbstractType
}

#[derive(Debug)]
pub struct FileDescription {
	pub statements: Vec<Statement>,
	pub functions: std::collections::HashMap<String, FunctionInfo>
}
