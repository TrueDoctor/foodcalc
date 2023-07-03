(function(scope){
'use strict';

function F(arity, fun, wrapper) {
  wrapper.a = arity;
  wrapper.f = fun;
  return wrapper;
}

function F2(fun) {
  return F(2, fun, function(a) { return function(b) { return fun(a,b); }; })
}
function F3(fun) {
  return F(3, fun, function(a) {
    return function(b) { return function(c) { return fun(a, b, c); }; };
  });
}
function F4(fun) {
  return F(4, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return fun(a, b, c, d); }; }; };
  });
}
function F5(fun) {
  return F(5, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return fun(a, b, c, d, e); }; }; }; };
  });
}
function F6(fun) {
  return F(6, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return function(f) {
    return fun(a, b, c, d, e, f); }; }; }; }; };
  });
}
function F7(fun) {
  return F(7, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return function(f) {
    return function(g) { return fun(a, b, c, d, e, f, g); }; }; }; }; }; };
  });
}
function F8(fun) {
  return F(8, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return function(f) {
    return function(g) { return function(h) {
    return fun(a, b, c, d, e, f, g, h); }; }; }; }; }; }; };
  });
}
function F9(fun) {
  return F(9, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return function(f) {
    return function(g) { return function(h) { return function(i) {
    return fun(a, b, c, d, e, f, g, h, i); }; }; }; }; }; }; }; };
  });
}

function A2(fun, a, b) {
  return fun.a === 2 ? fun.f(a, b) : fun(a)(b);
}
function A3(fun, a, b, c) {
  return fun.a === 3 ? fun.f(a, b, c) : fun(a)(b)(c);
}
function A4(fun, a, b, c, d) {
  return fun.a === 4 ? fun.f(a, b, c, d) : fun(a)(b)(c)(d);
}
function A5(fun, a, b, c, d, e) {
  return fun.a === 5 ? fun.f(a, b, c, d, e) : fun(a)(b)(c)(d)(e);
}
function A6(fun, a, b, c, d, e, f) {
  return fun.a === 6 ? fun.f(a, b, c, d, e, f) : fun(a)(b)(c)(d)(e)(f);
}
function A7(fun, a, b, c, d, e, f, g) {
  return fun.a === 7 ? fun.f(a, b, c, d, e, f, g) : fun(a)(b)(c)(d)(e)(f)(g);
}
function A8(fun, a, b, c, d, e, f, g, h) {
  return fun.a === 8 ? fun.f(a, b, c, d, e, f, g, h) : fun(a)(b)(c)(d)(e)(f)(g)(h);
}
function A9(fun, a, b, c, d, e, f, g, h, i) {
  return fun.a === 9 ? fun.f(a, b, c, d, e, f, g, h, i) : fun(a)(b)(c)(d)(e)(f)(g)(h)(i);
}

console.warn('Compiled in DEV mode. Follow the advice at https://elm-lang.org/0.19.1/optimize for better performance and smaller assets.');


// EQUALITY

function _Utils_eq(x, y)
{
	for (
		var pair, stack = [], isEqual = _Utils_eqHelp(x, y, 0, stack);
		isEqual && (pair = stack.pop());
		isEqual = _Utils_eqHelp(pair.a, pair.b, 0, stack)
		)
	{}

	return isEqual;
}

function _Utils_eqHelp(x, y, depth, stack)
{
	if (x === y)
	{
		return true;
	}

	if (typeof x !== 'object' || x === null || y === null)
	{
		typeof x === 'function' && _Debug_crash(5);
		return false;
	}

	if (depth > 100)
	{
		stack.push(_Utils_Tuple2(x,y));
		return true;
	}

	/**/
	if (x.$ === 'Set_elm_builtin')
	{
		x = $elm$core$Set$toList(x);
		y = $elm$core$Set$toList(y);
	}
	if (x.$ === 'RBNode_elm_builtin' || x.$ === 'RBEmpty_elm_builtin')
	{
		x = $elm$core$Dict$toList(x);
		y = $elm$core$Dict$toList(y);
	}
	//*/

	/**_UNUSED/
	if (x.$ < 0)
	{
		x = $elm$core$Dict$toList(x);
		y = $elm$core$Dict$toList(y);
	}
	//*/

	for (var key in x)
	{
		if (!_Utils_eqHelp(x[key], y[key], depth + 1, stack))
		{
			return false;
		}
	}
	return true;
}

var _Utils_equal = F2(_Utils_eq);
var _Utils_notEqual = F2(function(a, b) { return !_Utils_eq(a,b); });



// COMPARISONS

// Code in Generate/JavaScript.hs, Basics.js, and List.js depends on
// the particular integer values assigned to LT, EQ, and GT.

function _Utils_cmp(x, y, ord)
{
	if (typeof x !== 'object')
	{
		return x === y ? /*EQ*/ 0 : x < y ? /*LT*/ -1 : /*GT*/ 1;
	}

	/**/
	if (x instanceof String)
	{
		var a = x.valueOf();
		var b = y.valueOf();
		return a === b ? 0 : a < b ? -1 : 1;
	}
	//*/

	/**_UNUSED/
	if (typeof x.$ === 'undefined')
	//*/
	/**/
	if (x.$[0] === '#')
	//*/
	{
		return (ord = _Utils_cmp(x.a, y.a))
			? ord
			: (ord = _Utils_cmp(x.b, y.b))
				? ord
				: _Utils_cmp(x.c, y.c);
	}

	// traverse conses until end of a list or a mismatch
	for (; x.b && y.b && !(ord = _Utils_cmp(x.a, y.a)); x = x.b, y = y.b) {} // WHILE_CONSES
	return ord || (x.b ? /*GT*/ 1 : y.b ? /*LT*/ -1 : /*EQ*/ 0);
}

var _Utils_lt = F2(function(a, b) { return _Utils_cmp(a, b) < 0; });
var _Utils_le = F2(function(a, b) { return _Utils_cmp(a, b) < 1; });
var _Utils_gt = F2(function(a, b) { return _Utils_cmp(a, b) > 0; });
var _Utils_ge = F2(function(a, b) { return _Utils_cmp(a, b) >= 0; });

var _Utils_compare = F2(function(x, y)
{
	var n = _Utils_cmp(x, y);
	return n < 0 ? $elm$core$Basics$LT : n ? $elm$core$Basics$GT : $elm$core$Basics$EQ;
});


// COMMON VALUES

var _Utils_Tuple0_UNUSED = 0;
var _Utils_Tuple0 = { $: '#0' };

function _Utils_Tuple2_UNUSED(a, b) { return { a: a, b: b }; }
function _Utils_Tuple2(a, b) { return { $: '#2', a: a, b: b }; }

function _Utils_Tuple3_UNUSED(a, b, c) { return { a: a, b: b, c: c }; }
function _Utils_Tuple3(a, b, c) { return { $: '#3', a: a, b: b, c: c }; }

function _Utils_chr_UNUSED(c) { return c; }
function _Utils_chr(c) { return new String(c); }


// RECORDS

function _Utils_update(oldRecord, updatedFields)
{
	var newRecord = {};

	for (var key in oldRecord)
	{
		newRecord[key] = oldRecord[key];
	}

	for (var key in updatedFields)
	{
		newRecord[key] = updatedFields[key];
	}

	return newRecord;
}


// APPEND

var _Utils_append = F2(_Utils_ap);

function _Utils_ap(xs, ys)
{
	// append Strings
	if (typeof xs === 'string')
	{
		return xs + ys;
	}

	// append Lists
	if (!xs.b)
	{
		return ys;
	}
	var root = _List_Cons(xs.a, ys);
	xs = xs.b
	for (var curr = root; xs.b; xs = xs.b) // WHILE_CONS
	{
		curr = curr.b = _List_Cons(xs.a, ys);
	}
	return root;
}



var _List_Nil_UNUSED = { $: 0 };
var _List_Nil = { $: '[]' };

function _List_Cons_UNUSED(hd, tl) { return { $: 1, a: hd, b: tl }; }
function _List_Cons(hd, tl) { return { $: '::', a: hd, b: tl }; }


var _List_cons = F2(_List_Cons);

function _List_fromArray(arr)
{
	var out = _List_Nil;
	for (var i = arr.length; i--; )
	{
		out = _List_Cons(arr[i], out);
	}
	return out;
}

function _List_toArray(xs)
{
	for (var out = []; xs.b; xs = xs.b) // WHILE_CONS
	{
		out.push(xs.a);
	}
	return out;
}

var _List_map2 = F3(function(f, xs, ys)
{
	for (var arr = []; xs.b && ys.b; xs = xs.b, ys = ys.b) // WHILE_CONSES
	{
		arr.push(A2(f, xs.a, ys.a));
	}
	return _List_fromArray(arr);
});

var _List_map3 = F4(function(f, xs, ys, zs)
{
	for (var arr = []; xs.b && ys.b && zs.b; xs = xs.b, ys = ys.b, zs = zs.b) // WHILE_CONSES
	{
		arr.push(A3(f, xs.a, ys.a, zs.a));
	}
	return _List_fromArray(arr);
});

var _List_map4 = F5(function(f, ws, xs, ys, zs)
{
	for (var arr = []; ws.b && xs.b && ys.b && zs.b; ws = ws.b, xs = xs.b, ys = ys.b, zs = zs.b) // WHILE_CONSES
	{
		arr.push(A4(f, ws.a, xs.a, ys.a, zs.a));
	}
	return _List_fromArray(arr);
});

var _List_map5 = F6(function(f, vs, ws, xs, ys, zs)
{
	for (var arr = []; vs.b && ws.b && xs.b && ys.b && zs.b; vs = vs.b, ws = ws.b, xs = xs.b, ys = ys.b, zs = zs.b) // WHILE_CONSES
	{
		arr.push(A5(f, vs.a, ws.a, xs.a, ys.a, zs.a));
	}
	return _List_fromArray(arr);
});

var _List_sortBy = F2(function(f, xs)
{
	return _List_fromArray(_List_toArray(xs).sort(function(a, b) {
		return _Utils_cmp(f(a), f(b));
	}));
});

var _List_sortWith = F2(function(f, xs)
{
	return _List_fromArray(_List_toArray(xs).sort(function(a, b) {
		var ord = A2(f, a, b);
		return ord === $elm$core$Basics$EQ ? 0 : ord === $elm$core$Basics$LT ? -1 : 1;
	}));
});



var _JsArray_empty = [];

function _JsArray_singleton(value)
{
    return [value];
}

function _JsArray_length(array)
{
    return array.length;
}

var _JsArray_initialize = F3(function(size, offset, func)
{
    var result = new Array(size);

    for (var i = 0; i < size; i++)
    {
        result[i] = func(offset + i);
    }

    return result;
});

var _JsArray_initializeFromList = F2(function (max, ls)
{
    var result = new Array(max);

    for (var i = 0; i < max && ls.b; i++)
    {
        result[i] = ls.a;
        ls = ls.b;
    }

    result.length = i;
    return _Utils_Tuple2(result, ls);
});

var _JsArray_unsafeGet = F2(function(index, array)
{
    return array[index];
});

var _JsArray_unsafeSet = F3(function(index, value, array)
{
    var length = array.length;
    var result = new Array(length);

    for (var i = 0; i < length; i++)
    {
        result[i] = array[i];
    }

    result[index] = value;
    return result;
});

var _JsArray_push = F2(function(value, array)
{
    var length = array.length;
    var result = new Array(length + 1);

    for (var i = 0; i < length; i++)
    {
        result[i] = array[i];
    }

    result[length] = value;
    return result;
});

var _JsArray_foldl = F3(function(func, acc, array)
{
    var length = array.length;

    for (var i = 0; i < length; i++)
    {
        acc = A2(func, array[i], acc);
    }

    return acc;
});

var _JsArray_foldr = F3(function(func, acc, array)
{
    for (var i = array.length - 1; i >= 0; i--)
    {
        acc = A2(func, array[i], acc);
    }

    return acc;
});

var _JsArray_map = F2(function(func, array)
{
    var length = array.length;
    var result = new Array(length);

    for (var i = 0; i < length; i++)
    {
        result[i] = func(array[i]);
    }

    return result;
});

var _JsArray_indexedMap = F3(function(func, offset, array)
{
    var length = array.length;
    var result = new Array(length);

    for (var i = 0; i < length; i++)
    {
        result[i] = A2(func, offset + i, array[i]);
    }

    return result;
});

var _JsArray_slice = F3(function(from, to, array)
{
    return array.slice(from, to);
});

var _JsArray_appendN = F3(function(n, dest, source)
{
    var destLen = dest.length;
    var itemsToCopy = n - destLen;

    if (itemsToCopy > source.length)
    {
        itemsToCopy = source.length;
    }

    var size = destLen + itemsToCopy;
    var result = new Array(size);

    for (var i = 0; i < destLen; i++)
    {
        result[i] = dest[i];
    }

    for (var i = 0; i < itemsToCopy; i++)
    {
        result[i + destLen] = source[i];
    }

    return result;
});



// LOG

var _Debug_log_UNUSED = F2(function(tag, value)
{
	return value;
});

var _Debug_log = F2(function(tag, value)
{
	console.log(tag + ': ' + _Debug_toString(value));
	return value;
});


// TODOS

function _Debug_todo(moduleName, region)
{
	return function(message) {
		_Debug_crash(8, moduleName, region, message);
	};
}

function _Debug_todoCase(moduleName, region, value)
{
	return function(message) {
		_Debug_crash(9, moduleName, region, value, message);
	};
}


// TO STRING

function _Debug_toString_UNUSED(value)
{
	return '<internals>';
}

function _Debug_toString(value)
{
	return _Debug_toAnsiString(false, value);
}

function _Debug_toAnsiString(ansi, value)
{
	if (typeof value === 'function')
	{
		return _Debug_internalColor(ansi, '<function>');
	}

	if (typeof value === 'boolean')
	{
		return _Debug_ctorColor(ansi, value ? 'True' : 'False');
	}

	if (typeof value === 'number')
	{
		return _Debug_numberColor(ansi, value + '');
	}

	if (value instanceof String)
	{
		return _Debug_charColor(ansi, "'" + _Debug_addSlashes(value, true) + "'");
	}

	if (typeof value === 'string')
	{
		return _Debug_stringColor(ansi, '"' + _Debug_addSlashes(value, false) + '"');
	}

	if (typeof value === 'object' && '$' in value)
	{
		var tag = value.$;

		if (typeof tag === 'number')
		{
			return _Debug_internalColor(ansi, '<internals>');
		}

		if (tag[0] === '#')
		{
			var output = [];
			for (var k in value)
			{
				if (k === '$') continue;
				output.push(_Debug_toAnsiString(ansi, value[k]));
			}
			return '(' + output.join(',') + ')';
		}

		if (tag === 'Set_elm_builtin')
		{
			return _Debug_ctorColor(ansi, 'Set')
				+ _Debug_fadeColor(ansi, '.fromList') + ' '
				+ _Debug_toAnsiString(ansi, $elm$core$Set$toList(value));
		}

		if (tag === 'RBNode_elm_builtin' || tag === 'RBEmpty_elm_builtin')
		{
			return _Debug_ctorColor(ansi, 'Dict')
				+ _Debug_fadeColor(ansi, '.fromList') + ' '
				+ _Debug_toAnsiString(ansi, $elm$core$Dict$toList(value));
		}

		if (tag === 'Array_elm_builtin')
		{
			return _Debug_ctorColor(ansi, 'Array')
				+ _Debug_fadeColor(ansi, '.fromList') + ' '
				+ _Debug_toAnsiString(ansi, $elm$core$Array$toList(value));
		}

		if (tag === '::' || tag === '[]')
		{
			var output = '[';

			value.b && (output += _Debug_toAnsiString(ansi, value.a), value = value.b)

			for (; value.b; value = value.b) // WHILE_CONS
			{
				output += ',' + _Debug_toAnsiString(ansi, value.a);
			}
			return output + ']';
		}

		var output = '';
		for (var i in value)
		{
			if (i === '$') continue;
			var str = _Debug_toAnsiString(ansi, value[i]);
			var c0 = str[0];
			var parenless = c0 === '{' || c0 === '(' || c0 === '[' || c0 === '<' || c0 === '"' || str.indexOf(' ') < 0;
			output += ' ' + (parenless ? str : '(' + str + ')');
		}
		return _Debug_ctorColor(ansi, tag) + output;
	}

	if (typeof DataView === 'function' && value instanceof DataView)
	{
		return _Debug_stringColor(ansi, '<' + value.byteLength + ' bytes>');
	}

	if (typeof File !== 'undefined' && value instanceof File)
	{
		return _Debug_internalColor(ansi, '<' + value.name + '>');
	}

	if (typeof value === 'object')
	{
		var output = [];
		for (var key in value)
		{
			var field = key[0] === '_' ? key.slice(1) : key;
			output.push(_Debug_fadeColor(ansi, field) + ' = ' + _Debug_toAnsiString(ansi, value[key]));
		}
		if (output.length === 0)
		{
			return '{}';
		}
		return '{ ' + output.join(', ') + ' }';
	}

	return _Debug_internalColor(ansi, '<internals>');
}

function _Debug_addSlashes(str, isChar)
{
	var s = str
		.replace(/\\/g, '\\\\')
		.replace(/\n/g, '\\n')
		.replace(/\t/g, '\\t')
		.replace(/\r/g, '\\r')
		.replace(/\v/g, '\\v')
		.replace(/\0/g, '\\0');

	if (isChar)
	{
		return s.replace(/\'/g, '\\\'');
	}
	else
	{
		return s.replace(/\"/g, '\\"');
	}
}

function _Debug_ctorColor(ansi, string)
{
	return ansi ? '\x1b[96m' + string + '\x1b[0m' : string;
}

function _Debug_numberColor(ansi, string)
{
	return ansi ? '\x1b[95m' + string + '\x1b[0m' : string;
}

function _Debug_stringColor(ansi, string)
{
	return ansi ? '\x1b[93m' + string + '\x1b[0m' : string;
}

function _Debug_charColor(ansi, string)
{
	return ansi ? '\x1b[92m' + string + '\x1b[0m' : string;
}

function _Debug_fadeColor(ansi, string)
{
	return ansi ? '\x1b[37m' + string + '\x1b[0m' : string;
}

function _Debug_internalColor(ansi, string)
{
	return ansi ? '\x1b[36m' + string + '\x1b[0m' : string;
}

function _Debug_toHexDigit(n)
{
	return String.fromCharCode(n < 10 ? 48 + n : 55 + n);
}


// CRASH


function _Debug_crash_UNUSED(identifier)
{
	throw new Error('https://github.com/elm/core/blob/1.0.0/hints/' + identifier + '.md');
}


function _Debug_crash(identifier, fact1, fact2, fact3, fact4)
{
	switch(identifier)
	{
		case 0:
			throw new Error('What node should I take over? In JavaScript I need something like:\n\n    Elm.Main.init({\n        node: document.getElementById("elm-node")\n    })\n\nYou need to do this with any Browser.sandbox or Browser.element program.');

		case 1:
			throw new Error('Browser.application programs cannot handle URLs like this:\n\n    ' + document.location.href + '\n\nWhat is the root? The root of your file system? Try looking at this program with `elm reactor` or some other server.');

		case 2:
			var jsonErrorString = fact1;
			throw new Error('Problem with the flags given to your Elm program on initialization.\n\n' + jsonErrorString);

		case 3:
			var portName = fact1;
			throw new Error('There can only be one port named `' + portName + '`, but your program has multiple.');

		case 4:
			var portName = fact1;
			var problem = fact2;
			throw new Error('Trying to send an unexpected type of value through port `' + portName + '`:\n' + problem);

		case 5:
			throw new Error('Trying to use `(==)` on functions.\nThere is no way to know if functions are "the same" in the Elm sense.\nRead more about this at https://package.elm-lang.org/packages/elm/core/latest/Basics#== which describes why it is this way and what the better version will look like.');

		case 6:
			var moduleName = fact1;
			throw new Error('Your page is loading multiple Elm scripts with a module named ' + moduleName + '. Maybe a duplicate script is getting loaded accidentally? If not, rename one of them so I know which is which!');

		case 8:
			var moduleName = fact1;
			var region = fact2;
			var message = fact3;
			throw new Error('TODO in module `' + moduleName + '` ' + _Debug_regionToString(region) + '\n\n' + message);

		case 9:
			var moduleName = fact1;
			var region = fact2;
			var value = fact3;
			var message = fact4;
			throw new Error(
				'TODO in module `' + moduleName + '` from the `case` expression '
				+ _Debug_regionToString(region) + '\n\nIt received the following value:\n\n    '
				+ _Debug_toString(value).replace('\n', '\n    ')
				+ '\n\nBut the branch that handles it says:\n\n    ' + message.replace('\n', '\n    ')
			);

		case 10:
			throw new Error('Bug in https://github.com/elm/virtual-dom/issues');

		case 11:
			throw new Error('Cannot perform mod 0. Division by zero error.');
	}
}

function _Debug_regionToString(region)
{
	if (region.start.line === region.end.line)
	{
		return 'on line ' + region.start.line;
	}
	return 'on lines ' + region.start.line + ' through ' + region.end.line;
}



// MATH

var _Basics_add = F2(function(a, b) { return a + b; });
var _Basics_sub = F2(function(a, b) { return a - b; });
var _Basics_mul = F2(function(a, b) { return a * b; });
var _Basics_fdiv = F2(function(a, b) { return a / b; });
var _Basics_idiv = F2(function(a, b) { return (a / b) | 0; });
var _Basics_pow = F2(Math.pow);

var _Basics_remainderBy = F2(function(b, a) { return a % b; });

// https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/divmodnote-letter.pdf
var _Basics_modBy = F2(function(modulus, x)
{
	var answer = x % modulus;
	return modulus === 0
		? _Debug_crash(11)
		:
	((answer > 0 && modulus < 0) || (answer < 0 && modulus > 0))
		? answer + modulus
		: answer;
});


// TRIGONOMETRY

var _Basics_pi = Math.PI;
var _Basics_e = Math.E;
var _Basics_cos = Math.cos;
var _Basics_sin = Math.sin;
var _Basics_tan = Math.tan;
var _Basics_acos = Math.acos;
var _Basics_asin = Math.asin;
var _Basics_atan = Math.atan;
var _Basics_atan2 = F2(Math.atan2);


// MORE MATH

function _Basics_toFloat(x) { return x; }
function _Basics_truncate(n) { return n | 0; }
function _Basics_isInfinite(n) { return n === Infinity || n === -Infinity; }

var _Basics_ceiling = Math.ceil;
var _Basics_floor = Math.floor;
var _Basics_round = Math.round;
var _Basics_sqrt = Math.sqrt;
var _Basics_log = Math.log;
var _Basics_isNaN = isNaN;


// BOOLEANS

function _Basics_not(bool) { return !bool; }
var _Basics_and = F2(function(a, b) { return a && b; });
var _Basics_or  = F2(function(a, b) { return a || b; });
var _Basics_xor = F2(function(a, b) { return a !== b; });



var _String_cons = F2(function(chr, str)
{
	return chr + str;
});

function _String_uncons(string)
{
	var word = string.charCodeAt(0);
	return !isNaN(word)
		? $elm$core$Maybe$Just(
			0xD800 <= word && word <= 0xDBFF
				? _Utils_Tuple2(_Utils_chr(string[0] + string[1]), string.slice(2))
				: _Utils_Tuple2(_Utils_chr(string[0]), string.slice(1))
		)
		: $elm$core$Maybe$Nothing;
}

var _String_append = F2(function(a, b)
{
	return a + b;
});

function _String_length(str)
{
	return str.length;
}

var _String_map = F2(function(func, string)
{
	var len = string.length;
	var array = new Array(len);
	var i = 0;
	while (i < len)
	{
		var word = string.charCodeAt(i);
		if (0xD800 <= word && word <= 0xDBFF)
		{
			array[i] = func(_Utils_chr(string[i] + string[i+1]));
			i += 2;
			continue;
		}
		array[i] = func(_Utils_chr(string[i]));
		i++;
	}
	return array.join('');
});

var _String_filter = F2(function(isGood, str)
{
	var arr = [];
	var len = str.length;
	var i = 0;
	while (i < len)
	{
		var char = str[i];
		var word = str.charCodeAt(i);
		i++;
		if (0xD800 <= word && word <= 0xDBFF)
		{
			char += str[i];
			i++;
		}

		if (isGood(_Utils_chr(char)))
		{
			arr.push(char);
		}
	}
	return arr.join('');
});

function _String_reverse(str)
{
	var len = str.length;
	var arr = new Array(len);
	var i = 0;
	while (i < len)
	{
		var word = str.charCodeAt(i);
		if (0xD800 <= word && word <= 0xDBFF)
		{
			arr[len - i] = str[i + 1];
			i++;
			arr[len - i] = str[i - 1];
			i++;
		}
		else
		{
			arr[len - i] = str[i];
			i++;
		}
	}
	return arr.join('');
}

var _String_foldl = F3(function(func, state, string)
{
	var len = string.length;
	var i = 0;
	while (i < len)
	{
		var char = string[i];
		var word = string.charCodeAt(i);
		i++;
		if (0xD800 <= word && word <= 0xDBFF)
		{
			char += string[i];
			i++;
		}
		state = A2(func, _Utils_chr(char), state);
	}
	return state;
});

var _String_foldr = F3(function(func, state, string)
{
	var i = string.length;
	while (i--)
	{
		var char = string[i];
		var word = string.charCodeAt(i);
		if (0xDC00 <= word && word <= 0xDFFF)
		{
			i--;
			char = string[i] + char;
		}
		state = A2(func, _Utils_chr(char), state);
	}
	return state;
});

var _String_split = F2(function(sep, str)
{
	return str.split(sep);
});

var _String_join = F2(function(sep, strs)
{
	return strs.join(sep);
});

var _String_slice = F3(function(start, end, str) {
	return str.slice(start, end);
});

function _String_trim(str)
{
	return str.trim();
}

function _String_trimLeft(str)
{
	return str.replace(/^\s+/, '');
}

function _String_trimRight(str)
{
	return str.replace(/\s+$/, '');
}

function _String_words(str)
{
	return _List_fromArray(str.trim().split(/\s+/g));
}

function _String_lines(str)
{
	return _List_fromArray(str.split(/\r\n|\r|\n/g));
}

function _String_toUpper(str)
{
	return str.toUpperCase();
}

function _String_toLower(str)
{
	return str.toLowerCase();
}

var _String_any = F2(function(isGood, string)
{
	var i = string.length;
	while (i--)
	{
		var char = string[i];
		var word = string.charCodeAt(i);
		if (0xDC00 <= word && word <= 0xDFFF)
		{
			i--;
			char = string[i] + char;
		}
		if (isGood(_Utils_chr(char)))
		{
			return true;
		}
	}
	return false;
});

var _String_all = F2(function(isGood, string)
{
	var i = string.length;
	while (i--)
	{
		var char = string[i];
		var word = string.charCodeAt(i);
		if (0xDC00 <= word && word <= 0xDFFF)
		{
			i--;
			char = string[i] + char;
		}
		if (!isGood(_Utils_chr(char)))
		{
			return false;
		}
	}
	return true;
});

var _String_contains = F2(function(sub, str)
{
	return str.indexOf(sub) > -1;
});

var _String_startsWith = F2(function(sub, str)
{
	return str.indexOf(sub) === 0;
});

var _String_endsWith = F2(function(sub, str)
{
	return str.length >= sub.length &&
		str.lastIndexOf(sub) === str.length - sub.length;
});

var _String_indexes = F2(function(sub, str)
{
	var subLen = sub.length;

	if (subLen < 1)
	{
		return _List_Nil;
	}

	var i = 0;
	var is = [];

	while ((i = str.indexOf(sub, i)) > -1)
	{
		is.push(i);
		i = i + subLen;
	}

	return _List_fromArray(is);
});


// TO STRING

function _String_fromNumber(number)
{
	return number + '';
}


// INT CONVERSIONS

function _String_toInt(str)
{
	var total = 0;
	var code0 = str.charCodeAt(0);
	var start = code0 == 0x2B /* + */ || code0 == 0x2D /* - */ ? 1 : 0;

	for (var i = start; i < str.length; ++i)
	{
		var code = str.charCodeAt(i);
		if (code < 0x30 || 0x39 < code)
		{
			return $elm$core$Maybe$Nothing;
		}
		total = 10 * total + code - 0x30;
	}

	return i == start
		? $elm$core$Maybe$Nothing
		: $elm$core$Maybe$Just(code0 == 0x2D ? -total : total);
}


// FLOAT CONVERSIONS

function _String_toFloat(s)
{
	// check if it is a hex, octal, or binary number
	if (s.length === 0 || /[\sxbo]/.test(s))
	{
		return $elm$core$Maybe$Nothing;
	}
	var n = +s;
	// faster isNaN check
	return n === n ? $elm$core$Maybe$Just(n) : $elm$core$Maybe$Nothing;
}

function _String_fromList(chars)
{
	return _List_toArray(chars).join('');
}




function _Char_toCode(char)
{
	var code = char.charCodeAt(0);
	if (0xD800 <= code && code <= 0xDBFF)
	{
		return (code - 0xD800) * 0x400 + char.charCodeAt(1) - 0xDC00 + 0x10000
	}
	return code;
}

function _Char_fromCode(code)
{
	return _Utils_chr(
		(code < 0 || 0x10FFFF < code)
			? '\uFFFD'
			:
		(code <= 0xFFFF)
			? String.fromCharCode(code)
			:
		(code -= 0x10000,
			String.fromCharCode(Math.floor(code / 0x400) + 0xD800, code % 0x400 + 0xDC00)
		)
	);
}

function _Char_toUpper(char)
{
	return _Utils_chr(char.toUpperCase());
}

function _Char_toLower(char)
{
	return _Utils_chr(char.toLowerCase());
}

function _Char_toLocaleUpper(char)
{
	return _Utils_chr(char.toLocaleUpperCase());
}

function _Char_toLocaleLower(char)
{
	return _Utils_chr(char.toLocaleLowerCase());
}



/**/
function _Json_errorToString(error)
{
	return $elm$json$Json$Decode$errorToString(error);
}
//*/


// CORE DECODERS

function _Json_succeed(msg)
{
	return {
		$: 0,
		a: msg
	};
}

function _Json_fail(msg)
{
	return {
		$: 1,
		a: msg
	};
}

function _Json_decodePrim(decoder)
{
	return { $: 2, b: decoder };
}

var _Json_decodeInt = _Json_decodePrim(function(value) {
	return (typeof value !== 'number')
		? _Json_expecting('an INT', value)
		:
	(-2147483647 < value && value < 2147483647 && (value | 0) === value)
		? $elm$core$Result$Ok(value)
		:
	(isFinite(value) && !(value % 1))
		? $elm$core$Result$Ok(value)
		: _Json_expecting('an INT', value);
});

var _Json_decodeBool = _Json_decodePrim(function(value) {
	return (typeof value === 'boolean')
		? $elm$core$Result$Ok(value)
		: _Json_expecting('a BOOL', value);
});

var _Json_decodeFloat = _Json_decodePrim(function(value) {
	return (typeof value === 'number')
		? $elm$core$Result$Ok(value)
		: _Json_expecting('a FLOAT', value);
});

var _Json_decodeValue = _Json_decodePrim(function(value) {
	return $elm$core$Result$Ok(_Json_wrap(value));
});

var _Json_decodeString = _Json_decodePrim(function(value) {
	return (typeof value === 'string')
		? $elm$core$Result$Ok(value)
		: (value instanceof String)
			? $elm$core$Result$Ok(value + '')
			: _Json_expecting('a STRING', value);
});

function _Json_decodeList(decoder) { return { $: 3, b: decoder }; }
function _Json_decodeArray(decoder) { return { $: 4, b: decoder }; }

function _Json_decodeNull(value) { return { $: 5, c: value }; }

var _Json_decodeField = F2(function(field, decoder)
{
	return {
		$: 6,
		d: field,
		b: decoder
	};
});

var _Json_decodeIndex = F2(function(index, decoder)
{
	return {
		$: 7,
		e: index,
		b: decoder
	};
});

function _Json_decodeKeyValuePairs(decoder)
{
	return {
		$: 8,
		b: decoder
	};
}

function _Json_mapMany(f, decoders)
{
	return {
		$: 9,
		f: f,
		g: decoders
	};
}

var _Json_andThen = F2(function(callback, decoder)
{
	return {
		$: 10,
		b: decoder,
		h: callback
	};
});

function _Json_oneOf(decoders)
{
	return {
		$: 11,
		g: decoders
	};
}


// DECODING OBJECTS

var _Json_map1 = F2(function(f, d1)
{
	return _Json_mapMany(f, [d1]);
});

var _Json_map2 = F3(function(f, d1, d2)
{
	return _Json_mapMany(f, [d1, d2]);
});

var _Json_map3 = F4(function(f, d1, d2, d3)
{
	return _Json_mapMany(f, [d1, d2, d3]);
});

var _Json_map4 = F5(function(f, d1, d2, d3, d4)
{
	return _Json_mapMany(f, [d1, d2, d3, d4]);
});

var _Json_map5 = F6(function(f, d1, d2, d3, d4, d5)
{
	return _Json_mapMany(f, [d1, d2, d3, d4, d5]);
});

var _Json_map6 = F7(function(f, d1, d2, d3, d4, d5, d6)
{
	return _Json_mapMany(f, [d1, d2, d3, d4, d5, d6]);
});

var _Json_map7 = F8(function(f, d1, d2, d3, d4, d5, d6, d7)
{
	return _Json_mapMany(f, [d1, d2, d3, d4, d5, d6, d7]);
});

var _Json_map8 = F9(function(f, d1, d2, d3, d4, d5, d6, d7, d8)
{
	return _Json_mapMany(f, [d1, d2, d3, d4, d5, d6, d7, d8]);
});


// DECODE

var _Json_runOnString = F2(function(decoder, string)
{
	try
	{
		var value = JSON.parse(string);
		return _Json_runHelp(decoder, value);
	}
	catch (e)
	{
		return $elm$core$Result$Err(A2($elm$json$Json$Decode$Failure, 'This is not valid JSON! ' + e.message, _Json_wrap(string)));
	}
});

var _Json_run = F2(function(decoder, value)
{
	return _Json_runHelp(decoder, _Json_unwrap(value));
});

function _Json_runHelp(decoder, value)
{
	switch (decoder.$)
	{
		case 2:
			return decoder.b(value);

		case 5:
			return (value === null)
				? $elm$core$Result$Ok(decoder.c)
				: _Json_expecting('null', value);

		case 3:
			if (!_Json_isArray(value))
			{
				return _Json_expecting('a LIST', value);
			}
			return _Json_runArrayDecoder(decoder.b, value, _List_fromArray);

		case 4:
			if (!_Json_isArray(value))
			{
				return _Json_expecting('an ARRAY', value);
			}
			return _Json_runArrayDecoder(decoder.b, value, _Json_toElmArray);

		case 6:
			var field = decoder.d;
			if (typeof value !== 'object' || value === null || !(field in value))
			{
				return _Json_expecting('an OBJECT with a field named `' + field + '`', value);
			}
			var result = _Json_runHelp(decoder.b, value[field]);
			return ($elm$core$Result$isOk(result)) ? result : $elm$core$Result$Err(A2($elm$json$Json$Decode$Field, field, result.a));

		case 7:
			var index = decoder.e;
			if (!_Json_isArray(value))
			{
				return _Json_expecting('an ARRAY', value);
			}
			if (index >= value.length)
			{
				return _Json_expecting('a LONGER array. Need index ' + index + ' but only see ' + value.length + ' entries', value);
			}
			var result = _Json_runHelp(decoder.b, value[index]);
			return ($elm$core$Result$isOk(result)) ? result : $elm$core$Result$Err(A2($elm$json$Json$Decode$Index, index, result.a));

		case 8:
			if (typeof value !== 'object' || value === null || _Json_isArray(value))
			{
				return _Json_expecting('an OBJECT', value);
			}

			var keyValuePairs = _List_Nil;
			// TODO test perf of Object.keys and switch when support is good enough
			for (var key in value)
			{
				if (value.hasOwnProperty(key))
				{
					var result = _Json_runHelp(decoder.b, value[key]);
					if (!$elm$core$Result$isOk(result))
					{
						return $elm$core$Result$Err(A2($elm$json$Json$Decode$Field, key, result.a));
					}
					keyValuePairs = _List_Cons(_Utils_Tuple2(key, result.a), keyValuePairs);
				}
			}
			return $elm$core$Result$Ok($elm$core$List$reverse(keyValuePairs));

		case 9:
			var answer = decoder.f;
			var decoders = decoder.g;
			for (var i = 0; i < decoders.length; i++)
			{
				var result = _Json_runHelp(decoders[i], value);
				if (!$elm$core$Result$isOk(result))
				{
					return result;
				}
				answer = answer(result.a);
			}
			return $elm$core$Result$Ok(answer);

		case 10:
			var result = _Json_runHelp(decoder.b, value);
			return (!$elm$core$Result$isOk(result))
				? result
				: _Json_runHelp(decoder.h(result.a), value);

		case 11:
			var errors = _List_Nil;
			for (var temp = decoder.g; temp.b; temp = temp.b) // WHILE_CONS
			{
				var result = _Json_runHelp(temp.a, value);
				if ($elm$core$Result$isOk(result))
				{
					return result;
				}
				errors = _List_Cons(result.a, errors);
			}
			return $elm$core$Result$Err($elm$json$Json$Decode$OneOf($elm$core$List$reverse(errors)));

		case 1:
			return $elm$core$Result$Err(A2($elm$json$Json$Decode$Failure, decoder.a, _Json_wrap(value)));

		case 0:
			return $elm$core$Result$Ok(decoder.a);
	}
}

function _Json_runArrayDecoder(decoder, value, toElmValue)
{
	var len = value.length;
	var array = new Array(len);
	for (var i = 0; i < len; i++)
	{
		var result = _Json_runHelp(decoder, value[i]);
		if (!$elm$core$Result$isOk(result))
		{
			return $elm$core$Result$Err(A2($elm$json$Json$Decode$Index, i, result.a));
		}
		array[i] = result.a;
	}
	return $elm$core$Result$Ok(toElmValue(array));
}

function _Json_isArray(value)
{
	return Array.isArray(value) || (typeof FileList !== 'undefined' && value instanceof FileList);
}

function _Json_toElmArray(array)
{
	return A2($elm$core$Array$initialize, array.length, function(i) { return array[i]; });
}

function _Json_expecting(type, value)
{
	return $elm$core$Result$Err(A2($elm$json$Json$Decode$Failure, 'Expecting ' + type, _Json_wrap(value)));
}


// EQUALITY

function _Json_equality(x, y)
{
	if (x === y)
	{
		return true;
	}

	if (x.$ !== y.$)
	{
		return false;
	}

	switch (x.$)
	{
		case 0:
		case 1:
			return x.a === y.a;

		case 2:
			return x.b === y.b;

		case 5:
			return x.c === y.c;

		case 3:
		case 4:
		case 8:
			return _Json_equality(x.b, y.b);

		case 6:
			return x.d === y.d && _Json_equality(x.b, y.b);

		case 7:
			return x.e === y.e && _Json_equality(x.b, y.b);

		case 9:
			return x.f === y.f && _Json_listEquality(x.g, y.g);

		case 10:
			return x.h === y.h && _Json_equality(x.b, y.b);

		case 11:
			return _Json_listEquality(x.g, y.g);
	}
}

function _Json_listEquality(aDecoders, bDecoders)
{
	var len = aDecoders.length;
	if (len !== bDecoders.length)
	{
		return false;
	}
	for (var i = 0; i < len; i++)
	{
		if (!_Json_equality(aDecoders[i], bDecoders[i]))
		{
			return false;
		}
	}
	return true;
}


// ENCODE

var _Json_encode = F2(function(indentLevel, value)
{
	return JSON.stringify(_Json_unwrap(value), null, indentLevel) + '';
});

function _Json_wrap(value) { return { $: 0, a: value }; }
function _Json_unwrap(value) { return value.a; }

function _Json_wrap_UNUSED(value) { return value; }
function _Json_unwrap_UNUSED(value) { return value; }

function _Json_emptyArray() { return []; }
function _Json_emptyObject() { return {}; }

var _Json_addField = F3(function(key, value, object)
{
	object[key] = _Json_unwrap(value);
	return object;
});

function _Json_addEntry(func)
{
	return F2(function(entry, array)
	{
		array.push(_Json_unwrap(func(entry)));
		return array;
	});
}

var _Json_encodeNull = _Json_wrap(null);



// TASKS

function _Scheduler_succeed(value)
{
	return {
		$: 0,
		a: value
	};
}

function _Scheduler_fail(error)
{
	return {
		$: 1,
		a: error
	};
}

function _Scheduler_binding(callback)
{
	return {
		$: 2,
		b: callback,
		c: null
	};
}

var _Scheduler_andThen = F2(function(callback, task)
{
	return {
		$: 3,
		b: callback,
		d: task
	};
});

var _Scheduler_onError = F2(function(callback, task)
{
	return {
		$: 4,
		b: callback,
		d: task
	};
});

function _Scheduler_receive(callback)
{
	return {
		$: 5,
		b: callback
	};
}


// PROCESSES

var _Scheduler_guid = 0;

function _Scheduler_rawSpawn(task)
{
	var proc = {
		$: 0,
		e: _Scheduler_guid++,
		f: task,
		g: null,
		h: []
	};

	_Scheduler_enqueue(proc);

	return proc;
}

function _Scheduler_spawn(task)
{
	return _Scheduler_binding(function(callback) {
		callback(_Scheduler_succeed(_Scheduler_rawSpawn(task)));
	});
}

function _Scheduler_rawSend(proc, msg)
{
	proc.h.push(msg);
	_Scheduler_enqueue(proc);
}

var _Scheduler_send = F2(function(proc, msg)
{
	return _Scheduler_binding(function(callback) {
		_Scheduler_rawSend(proc, msg);
		callback(_Scheduler_succeed(_Utils_Tuple0));
	});
});

function _Scheduler_kill(proc)
{
	return _Scheduler_binding(function(callback) {
		var task = proc.f;
		if (task.$ === 2 && task.c)
		{
			task.c();
		}

		proc.f = null;

		callback(_Scheduler_succeed(_Utils_Tuple0));
	});
}


/* STEP PROCESSES

type alias Process =
  { $ : tag
  , id : unique_id
  , root : Task
  , stack : null | { $: SUCCEED | FAIL, a: callback, b: stack }
  , mailbox : [msg]
  }

*/


var _Scheduler_working = false;
var _Scheduler_queue = [];


function _Scheduler_enqueue(proc)
{
	_Scheduler_queue.push(proc);
	if (_Scheduler_working)
	{
		return;
	}
	_Scheduler_working = true;
	while (proc = _Scheduler_queue.shift())
	{
		_Scheduler_step(proc);
	}
	_Scheduler_working = false;
}


function _Scheduler_step(proc)
{
	while (proc.f)
	{
		var rootTag = proc.f.$;
		if (rootTag === 0 || rootTag === 1)
		{
			while (proc.g && proc.g.$ !== rootTag)
			{
				proc.g = proc.g.i;
			}
			if (!proc.g)
			{
				return;
			}
			proc.f = proc.g.b(proc.f.a);
			proc.g = proc.g.i;
		}
		else if (rootTag === 2)
		{
			proc.f.c = proc.f.b(function(newRoot) {
				proc.f = newRoot;
				_Scheduler_enqueue(proc);
			});
			return;
		}
		else if (rootTag === 5)
		{
			if (proc.h.length === 0)
			{
				return;
			}
			proc.f = proc.f.b(proc.h.shift());
		}
		else // if (rootTag === 3 || rootTag === 4)
		{
			proc.g = {
				$: rootTag === 3 ? 0 : 1,
				b: proc.f.b,
				i: proc.g
			};
			proc.f = proc.f.d;
		}
	}
}



function _Process_sleep(time)
{
	return _Scheduler_binding(function(callback) {
		var id = setTimeout(function() {
			callback(_Scheduler_succeed(_Utils_Tuple0));
		}, time);

		return function() { clearTimeout(id); };
	});
}




// PROGRAMS


var _Platform_worker = F4(function(impl, flagDecoder, debugMetadata, args)
{
	return _Platform_initialize(
		flagDecoder,
		args,
		impl.init,
		impl.update,
		impl.subscriptions,
		function() { return function() {} }
	);
});



// INITIALIZE A PROGRAM


function _Platform_initialize(flagDecoder, args, init, update, subscriptions, stepperBuilder)
{
	var result = A2(_Json_run, flagDecoder, _Json_wrap(args ? args['flags'] : undefined));
	$elm$core$Result$isOk(result) || _Debug_crash(2 /**/, _Json_errorToString(result.a) /**/);
	var managers = {};
	var initPair = init(result.a);
	var model = initPair.a;
	var stepper = stepperBuilder(sendToApp, model);
	var ports = _Platform_setupEffects(managers, sendToApp);

	function sendToApp(msg, viewMetadata)
	{
		var pair = A2(update, msg, model);
		stepper(model = pair.a, viewMetadata);
		_Platform_enqueueEffects(managers, pair.b, subscriptions(model));
	}

	_Platform_enqueueEffects(managers, initPair.b, subscriptions(model));

	return ports ? { ports: ports } : {};
}



// TRACK PRELOADS
//
// This is used by code in elm/browser and elm/http
// to register any HTTP requests that are triggered by init.
//


var _Platform_preload;


function _Platform_registerPreload(url)
{
	_Platform_preload.add(url);
}



// EFFECT MANAGERS


var _Platform_effectManagers = {};


function _Platform_setupEffects(managers, sendToApp)
{
	var ports;

	// setup all necessary effect managers
	for (var key in _Platform_effectManagers)
	{
		var manager = _Platform_effectManagers[key];

		if (manager.a)
		{
			ports = ports || {};
			ports[key] = manager.a(key, sendToApp);
		}

		managers[key] = _Platform_instantiateManager(manager, sendToApp);
	}

	return ports;
}


function _Platform_createManager(init, onEffects, onSelfMsg, cmdMap, subMap)
{
	return {
		b: init,
		c: onEffects,
		d: onSelfMsg,
		e: cmdMap,
		f: subMap
	};
}


function _Platform_instantiateManager(info, sendToApp)
{
	var router = {
		g: sendToApp,
		h: undefined
	};

	var onEffects = info.c;
	var onSelfMsg = info.d;
	var cmdMap = info.e;
	var subMap = info.f;

	function loop(state)
	{
		return A2(_Scheduler_andThen, loop, _Scheduler_receive(function(msg)
		{
			var value = msg.a;

			if (msg.$ === 0)
			{
				return A3(onSelfMsg, router, value, state);
			}

			return cmdMap && subMap
				? A4(onEffects, router, value.i, value.j, state)
				: A3(onEffects, router, cmdMap ? value.i : value.j, state);
		}));
	}

	return router.h = _Scheduler_rawSpawn(A2(_Scheduler_andThen, loop, info.b));
}



// ROUTING


var _Platform_sendToApp = F2(function(router, msg)
{
	return _Scheduler_binding(function(callback)
	{
		router.g(msg);
		callback(_Scheduler_succeed(_Utils_Tuple0));
	});
});


var _Platform_sendToSelf = F2(function(router, msg)
{
	return A2(_Scheduler_send, router.h, {
		$: 0,
		a: msg
	});
});



// BAGS


function _Platform_leaf(home)
{
	return function(value)
	{
		return {
			$: 1,
			k: home,
			l: value
		};
	};
}


function _Platform_batch(list)
{
	return {
		$: 2,
		m: list
	};
}


var _Platform_map = F2(function(tagger, bag)
{
	return {
		$: 3,
		n: tagger,
		o: bag
	}
});



// PIPE BAGS INTO EFFECT MANAGERS
//
// Effects must be queued!
//
// Say your init contains a synchronous command, like Time.now or Time.here
//
//   - This will produce a batch of effects (FX_1)
//   - The synchronous task triggers the subsequent `update` call
//   - This will produce a batch of effects (FX_2)
//
// If we just start dispatching FX_2, subscriptions from FX_2 can be processed
// before subscriptions from FX_1. No good! Earlier versions of this code had
// this problem, leading to these reports:
//
//   https://github.com/elm/core/issues/980
//   https://github.com/elm/core/pull/981
//   https://github.com/elm/compiler/issues/1776
//
// The queue is necessary to avoid ordering issues for synchronous commands.


// Why use true/false here? Why not just check the length of the queue?
// The goal is to detect "are we currently dispatching effects?" If we
// are, we need to bail and let the ongoing while loop handle things.
//
// Now say the queue has 1 element. When we dequeue the final element,
// the queue will be empty, but we are still actively dispatching effects.
// So you could get queue jumping in a really tricky category of cases.
//
var _Platform_effectsQueue = [];
var _Platform_effectsActive = false;


function _Platform_enqueueEffects(managers, cmdBag, subBag)
{
	_Platform_effectsQueue.push({ p: managers, q: cmdBag, r: subBag });

	if (_Platform_effectsActive) return;

	_Platform_effectsActive = true;
	for (var fx; fx = _Platform_effectsQueue.shift(); )
	{
		_Platform_dispatchEffects(fx.p, fx.q, fx.r);
	}
	_Platform_effectsActive = false;
}


function _Platform_dispatchEffects(managers, cmdBag, subBag)
{
	var effectsDict = {};
	_Platform_gatherEffects(true, cmdBag, effectsDict, null);
	_Platform_gatherEffects(false, subBag, effectsDict, null);

	for (var home in managers)
	{
		_Scheduler_rawSend(managers[home], {
			$: 'fx',
			a: effectsDict[home] || { i: _List_Nil, j: _List_Nil }
		});
	}
}


function _Platform_gatherEffects(isCmd, bag, effectsDict, taggers)
{
	switch (bag.$)
	{
		case 1:
			var home = bag.k;
			var effect = _Platform_toEffect(isCmd, home, taggers, bag.l);
			effectsDict[home] = _Platform_insert(isCmd, effect, effectsDict[home]);
			return;

		case 2:
			for (var list = bag.m; list.b; list = list.b) // WHILE_CONS
			{
				_Platform_gatherEffects(isCmd, list.a, effectsDict, taggers);
			}
			return;

		case 3:
			_Platform_gatherEffects(isCmd, bag.o, effectsDict, {
				s: bag.n,
				t: taggers
			});
			return;
	}
}


function _Platform_toEffect(isCmd, home, taggers, value)
{
	function applyTaggers(x)
	{
		for (var temp = taggers; temp; temp = temp.t)
		{
			x = temp.s(x);
		}
		return x;
	}

	var map = isCmd
		? _Platform_effectManagers[home].e
		: _Platform_effectManagers[home].f;

	return A2(map, applyTaggers, value)
}


function _Platform_insert(isCmd, newEffect, effects)
{
	effects = effects || { i: _List_Nil, j: _List_Nil };

	isCmd
		? (effects.i = _List_Cons(newEffect, effects.i))
		: (effects.j = _List_Cons(newEffect, effects.j));

	return effects;
}



// PORTS


function _Platform_checkPortName(name)
{
	if (_Platform_effectManagers[name])
	{
		_Debug_crash(3, name)
	}
}



// OUTGOING PORTS


function _Platform_outgoingPort(name, converter)
{
	_Platform_checkPortName(name);
	_Platform_effectManagers[name] = {
		e: _Platform_outgoingPortMap,
		u: converter,
		a: _Platform_setupOutgoingPort
	};
	return _Platform_leaf(name);
}


var _Platform_outgoingPortMap = F2(function(tagger, value) { return value; });


function _Platform_setupOutgoingPort(name)
{
	var subs = [];
	var converter = _Platform_effectManagers[name].u;

	// CREATE MANAGER

	var init = _Process_sleep(0);

	_Platform_effectManagers[name].b = init;
	_Platform_effectManagers[name].c = F3(function(router, cmdList, state)
	{
		for ( ; cmdList.b; cmdList = cmdList.b) // WHILE_CONS
		{
			// grab a separate reference to subs in case unsubscribe is called
			var currentSubs = subs;
			var value = _Json_unwrap(converter(cmdList.a));
			for (var i = 0; i < currentSubs.length; i++)
			{
				currentSubs[i](value);
			}
		}
		return init;
	});

	// PUBLIC API

	function subscribe(callback)
	{
		subs.push(callback);
	}

	function unsubscribe(callback)
	{
		// copy subs into a new array in case unsubscribe is called within a
		// subscribed callback
		subs = subs.slice();
		var index = subs.indexOf(callback);
		if (index >= 0)
		{
			subs.splice(index, 1);
		}
	}

	return {
		subscribe: subscribe,
		unsubscribe: unsubscribe
	};
}



// INCOMING PORTS


function _Platform_incomingPort(name, converter)
{
	_Platform_checkPortName(name);
	_Platform_effectManagers[name] = {
		f: _Platform_incomingPortMap,
		u: converter,
		a: _Platform_setupIncomingPort
	};
	return _Platform_leaf(name);
}


var _Platform_incomingPortMap = F2(function(tagger, finalTagger)
{
	return function(value)
	{
		return tagger(finalTagger(value));
	};
});


function _Platform_setupIncomingPort(name, sendToApp)
{
	var subs = _List_Nil;
	var converter = _Platform_effectManagers[name].u;

	// CREATE MANAGER

	var init = _Scheduler_succeed(null);

	_Platform_effectManagers[name].b = init;
	_Platform_effectManagers[name].c = F3(function(router, subList, state)
	{
		subs = subList;
		return init;
	});

	// PUBLIC API

	function send(incomingValue)
	{
		var result = A2(_Json_run, converter, _Json_wrap(incomingValue));

		$elm$core$Result$isOk(result) || _Debug_crash(4, name, result.a);

		var value = result.a;
		for (var temp = subs; temp.b; temp = temp.b) // WHILE_CONS
		{
			sendToApp(temp.a(value));
		}
	}

	return { send: send };
}



// EXPORT ELM MODULES
//
// Have DEBUG and PROD versions so that we can (1) give nicer errors in
// debug mode and (2) not pay for the bits needed for that in prod mode.
//


function _Platform_export_UNUSED(exports)
{
	scope['Elm']
		? _Platform_mergeExportsProd(scope['Elm'], exports)
		: scope['Elm'] = exports;
}


function _Platform_mergeExportsProd(obj, exports)
{
	for (var name in exports)
	{
		(name in obj)
			? (name == 'init')
				? _Debug_crash(6)
				: _Platform_mergeExportsProd(obj[name], exports[name])
			: (obj[name] = exports[name]);
	}
}


function _Platform_export(exports)
{
	scope['Elm']
		? _Platform_mergeExportsDebug('Elm', scope['Elm'], exports)
		: scope['Elm'] = exports;
}


function _Platform_mergeExportsDebug(moduleName, obj, exports)
{
	for (var name in exports)
	{
		(name in obj)
			? (name == 'init')
				? _Debug_crash(6, moduleName)
				: _Platform_mergeExportsDebug(moduleName + '.' + name, obj[name], exports[name])
			: (obj[name] = exports[name]);
	}
}




// HELPERS


var _VirtualDom_divertHrefToApp;

var _VirtualDom_doc = typeof document !== 'undefined' ? document : {};


function _VirtualDom_appendChild(parent, child)
{
	parent.appendChild(child);
}

var _VirtualDom_init = F4(function(virtualNode, flagDecoder, debugMetadata, args)
{
	// NOTE: this function needs _Platform_export available to work

	/**_UNUSED/
	var node = args['node'];
	//*/
	/**/
	var node = args && args['node'] ? args['node'] : _Debug_crash(0);
	//*/

	node.parentNode.replaceChild(
		_VirtualDom_render(virtualNode, function() {}),
		node
	);

	return {};
});



// TEXT


function _VirtualDom_text(string)
{
	return {
		$: 0,
		a: string
	};
}



// NODE


var _VirtualDom_nodeNS = F2(function(namespace, tag)
{
	return F2(function(factList, kidList)
	{
		for (var kids = [], descendantsCount = 0; kidList.b; kidList = kidList.b) // WHILE_CONS
		{
			var kid = kidList.a;
			descendantsCount += (kid.b || 0);
			kids.push(kid);
		}
		descendantsCount += kids.length;

		return {
			$: 1,
			c: tag,
			d: _VirtualDom_organizeFacts(factList),
			e: kids,
			f: namespace,
			b: descendantsCount
		};
	});
});


var _VirtualDom_node = _VirtualDom_nodeNS(undefined);



// KEYED NODE


var _VirtualDom_keyedNodeNS = F2(function(namespace, tag)
{
	return F2(function(factList, kidList)
	{
		for (var kids = [], descendantsCount = 0; kidList.b; kidList = kidList.b) // WHILE_CONS
		{
			var kid = kidList.a;
			descendantsCount += (kid.b.b || 0);
			kids.push(kid);
		}
		descendantsCount += kids.length;

		return {
			$: 2,
			c: tag,
			d: _VirtualDom_organizeFacts(factList),
			e: kids,
			f: namespace,
			b: descendantsCount
		};
	});
});


var _VirtualDom_keyedNode = _VirtualDom_keyedNodeNS(undefined);



// CUSTOM


function _VirtualDom_custom(factList, model, render, diff)
{
	return {
		$: 3,
		d: _VirtualDom_organizeFacts(factList),
		g: model,
		h: render,
		i: diff
	};
}



// MAP


var _VirtualDom_map = F2(function(tagger, node)
{
	return {
		$: 4,
		j: tagger,
		k: node,
		b: 1 + (node.b || 0)
	};
});



// LAZY


function _VirtualDom_thunk(refs, thunk)
{
	return {
		$: 5,
		l: refs,
		m: thunk,
		k: undefined
	};
}

var _VirtualDom_lazy = F2(function(func, a)
{
	return _VirtualDom_thunk([func, a], function() {
		return func(a);
	});
});

var _VirtualDom_lazy2 = F3(function(func, a, b)
{
	return _VirtualDom_thunk([func, a, b], function() {
		return A2(func, a, b);
	});
});

var _VirtualDom_lazy3 = F4(function(func, a, b, c)
{
	return _VirtualDom_thunk([func, a, b, c], function() {
		return A3(func, a, b, c);
	});
});

var _VirtualDom_lazy4 = F5(function(func, a, b, c, d)
{
	return _VirtualDom_thunk([func, a, b, c, d], function() {
		return A4(func, a, b, c, d);
	});
});

var _VirtualDom_lazy5 = F6(function(func, a, b, c, d, e)
{
	return _VirtualDom_thunk([func, a, b, c, d, e], function() {
		return A5(func, a, b, c, d, e);
	});
});

var _VirtualDom_lazy6 = F7(function(func, a, b, c, d, e, f)
{
	return _VirtualDom_thunk([func, a, b, c, d, e, f], function() {
		return A6(func, a, b, c, d, e, f);
	});
});

var _VirtualDom_lazy7 = F8(function(func, a, b, c, d, e, f, g)
{
	return _VirtualDom_thunk([func, a, b, c, d, e, f, g], function() {
		return A7(func, a, b, c, d, e, f, g);
	});
});

var _VirtualDom_lazy8 = F9(function(func, a, b, c, d, e, f, g, h)
{
	return _VirtualDom_thunk([func, a, b, c, d, e, f, g, h], function() {
		return A8(func, a, b, c, d, e, f, g, h);
	});
});



// FACTS


var _VirtualDom_on = F2(function(key, handler)
{
	return {
		$: 'a0',
		n: key,
		o: handler
	};
});
var _VirtualDom_style = F2(function(key, value)
{
	return {
		$: 'a1',
		n: key,
		o: value
	};
});
var _VirtualDom_property = F2(function(key, value)
{
	return {
		$: 'a2',
		n: key,
		o: value
	};
});
var _VirtualDom_attribute = F2(function(key, value)
{
	return {
		$: 'a3',
		n: key,
		o: value
	};
});
var _VirtualDom_attributeNS = F3(function(namespace, key, value)
{
	return {
		$: 'a4',
		n: key,
		o: { f: namespace, o: value }
	};
});



// XSS ATTACK VECTOR CHECKS
//
// For some reason, tabs can appear in href protocols and it still works.
// So '\tjava\tSCRIPT:alert("!!!")' and 'javascript:alert("!!!")' are the same
// in practice. That is why _VirtualDom_RE_js and _VirtualDom_RE_js_html look
// so freaky.
//
// Pulling the regular expressions out to the top level gives a slight speed
// boost in small benchmarks (4-10%) but hoisting values to reduce allocation
// can be unpredictable in large programs where JIT may have a harder time with
// functions are not fully self-contained. The benefit is more that the js and
// js_html ones are so weird that I prefer to see them near each other.


var _VirtualDom_RE_script = /^script$/i;
var _VirtualDom_RE_on_formAction = /^(on|formAction$)/i;
var _VirtualDom_RE_js = /^\s*j\s*a\s*v\s*a\s*s\s*c\s*r\s*i\s*p\s*t\s*:/i;
var _VirtualDom_RE_js_html = /^\s*(j\s*a\s*v\s*a\s*s\s*c\s*r\s*i\s*p\s*t\s*:|d\s*a\s*t\s*a\s*:\s*t\s*e\s*x\s*t\s*\/\s*h\s*t\s*m\s*l\s*(,|;))/i;


function _VirtualDom_noScript(tag)
{
	return _VirtualDom_RE_script.test(tag) ? 'p' : tag;
}

function _VirtualDom_noOnOrFormAction(key)
{
	return _VirtualDom_RE_on_formAction.test(key) ? 'data-' + key : key;
}

function _VirtualDom_noInnerHtmlOrFormAction(key)
{
	return key == 'innerHTML' || key == 'formAction' ? 'data-' + key : key;
}

function _VirtualDom_noJavaScriptUri(value)
{
	return _VirtualDom_RE_js.test(value)
		? /**_UNUSED/''//*//**/'javascript:alert("This is an XSS vector. Please use ports or web components instead.")'//*/
		: value;
}

function _VirtualDom_noJavaScriptOrHtmlUri(value)
{
	return _VirtualDom_RE_js_html.test(value)
		? /**_UNUSED/''//*//**/'javascript:alert("This is an XSS vector. Please use ports or web components instead.")'//*/
		: value;
}

function _VirtualDom_noJavaScriptOrHtmlJson(value)
{
	return (typeof _Json_unwrap(value) === 'string' && _VirtualDom_RE_js_html.test(_Json_unwrap(value)))
		? _Json_wrap(
			/**_UNUSED/''//*//**/'javascript:alert("This is an XSS vector. Please use ports or web components instead.")'//*/
		) : value;
}



// MAP FACTS


var _VirtualDom_mapAttribute = F2(function(func, attr)
{
	return (attr.$ === 'a0')
		? A2(_VirtualDom_on, attr.n, _VirtualDom_mapHandler(func, attr.o))
		: attr;
});

function _VirtualDom_mapHandler(func, handler)
{
	var tag = $elm$virtual_dom$VirtualDom$toHandlerInt(handler);

	// 0 = Normal
	// 1 = MayStopPropagation
	// 2 = MayPreventDefault
	// 3 = Custom

	return {
		$: handler.$,
		a:
			!tag
				? A2($elm$json$Json$Decode$map, func, handler.a)
				:
			A3($elm$json$Json$Decode$map2,
				tag < 3
					? _VirtualDom_mapEventTuple
					: _VirtualDom_mapEventRecord,
				$elm$json$Json$Decode$succeed(func),
				handler.a
			)
	};
}

var _VirtualDom_mapEventTuple = F2(function(func, tuple)
{
	return _Utils_Tuple2(func(tuple.a), tuple.b);
});

var _VirtualDom_mapEventRecord = F2(function(func, record)
{
	return {
		message: func(record.message),
		stopPropagation: record.stopPropagation,
		preventDefault: record.preventDefault
	}
});



// ORGANIZE FACTS


function _VirtualDom_organizeFacts(factList)
{
	for (var facts = {}; factList.b; factList = factList.b) // WHILE_CONS
	{
		var entry = factList.a;

		var tag = entry.$;
		var key = entry.n;
		var value = entry.o;

		if (tag === 'a2')
		{
			(key === 'className')
				? _VirtualDom_addClass(facts, key, _Json_unwrap(value))
				: facts[key] = _Json_unwrap(value);

			continue;
		}

		var subFacts = facts[tag] || (facts[tag] = {});
		(tag === 'a3' && key === 'class')
			? _VirtualDom_addClass(subFacts, key, value)
			: subFacts[key] = value;
	}

	return facts;
}

function _VirtualDom_addClass(object, key, newClass)
{
	var classes = object[key];
	object[key] = classes ? classes + ' ' + newClass : newClass;
}



// RENDER


function _VirtualDom_render(vNode, eventNode)
{
	var tag = vNode.$;

	if (tag === 5)
	{
		return _VirtualDom_render(vNode.k || (vNode.k = vNode.m()), eventNode);
	}

	if (tag === 0)
	{
		return _VirtualDom_doc.createTextNode(vNode.a);
	}

	if (tag === 4)
	{
		var subNode = vNode.k;
		var tagger = vNode.j;

		while (subNode.$ === 4)
		{
			typeof tagger !== 'object'
				? tagger = [tagger, subNode.j]
				: tagger.push(subNode.j);

			subNode = subNode.k;
		}

		var subEventRoot = { j: tagger, p: eventNode };
		var domNode = _VirtualDom_render(subNode, subEventRoot);
		domNode.elm_event_node_ref = subEventRoot;
		return domNode;
	}

	if (tag === 3)
	{
		var domNode = vNode.h(vNode.g);
		_VirtualDom_applyFacts(domNode, eventNode, vNode.d);
		return domNode;
	}

	// at this point `tag` must be 1 or 2

	var domNode = vNode.f
		? _VirtualDom_doc.createElementNS(vNode.f, vNode.c)
		: _VirtualDom_doc.createElement(vNode.c);

	if (_VirtualDom_divertHrefToApp && vNode.c == 'a')
	{
		domNode.addEventListener('click', _VirtualDom_divertHrefToApp(domNode));
	}

	_VirtualDom_applyFacts(domNode, eventNode, vNode.d);

	for (var kids = vNode.e, i = 0; i < kids.length; i++)
	{
		_VirtualDom_appendChild(domNode, _VirtualDom_render(tag === 1 ? kids[i] : kids[i].b, eventNode));
	}

	return domNode;
}



// APPLY FACTS


function _VirtualDom_applyFacts(domNode, eventNode, facts)
{
	for (var key in facts)
	{
		var value = facts[key];

		key === 'a1'
			? _VirtualDom_applyStyles(domNode, value)
			:
		key === 'a0'
			? _VirtualDom_applyEvents(domNode, eventNode, value)
			:
		key === 'a3'
			? _VirtualDom_applyAttrs(domNode, value)
			:
		key === 'a4'
			? _VirtualDom_applyAttrsNS(domNode, value)
			:
		((key !== 'value' && key !== 'checked') || domNode[key] !== value) && (domNode[key] = value);
	}
}



// APPLY STYLES


function _VirtualDom_applyStyles(domNode, styles)
{
	var domNodeStyle = domNode.style;

	for (var key in styles)
	{
		domNodeStyle[key] = styles[key];
	}
}



// APPLY ATTRS


function _VirtualDom_applyAttrs(domNode, attrs)
{
	for (var key in attrs)
	{
		var value = attrs[key];
		typeof value !== 'undefined'
			? domNode.setAttribute(key, value)
			: domNode.removeAttribute(key);
	}
}



// APPLY NAMESPACED ATTRS


function _VirtualDom_applyAttrsNS(domNode, nsAttrs)
{
	for (var key in nsAttrs)
	{
		var pair = nsAttrs[key];
		var namespace = pair.f;
		var value = pair.o;

		typeof value !== 'undefined'
			? domNode.setAttributeNS(namespace, key, value)
			: domNode.removeAttributeNS(namespace, key);
	}
}



// APPLY EVENTS


function _VirtualDom_applyEvents(domNode, eventNode, events)
{
	var allCallbacks = domNode.elmFs || (domNode.elmFs = {});

	for (var key in events)
	{
		var newHandler = events[key];
		var oldCallback = allCallbacks[key];

		if (!newHandler)
		{
			domNode.removeEventListener(key, oldCallback);
			allCallbacks[key] = undefined;
			continue;
		}

		if (oldCallback)
		{
			var oldHandler = oldCallback.q;
			if (oldHandler.$ === newHandler.$)
			{
				oldCallback.q = newHandler;
				continue;
			}
			domNode.removeEventListener(key, oldCallback);
		}

		oldCallback = _VirtualDom_makeCallback(eventNode, newHandler);
		domNode.addEventListener(key, oldCallback,
			_VirtualDom_passiveSupported
			&& { passive: $elm$virtual_dom$VirtualDom$toHandlerInt(newHandler) < 2 }
		);
		allCallbacks[key] = oldCallback;
	}
}



// PASSIVE EVENTS


var _VirtualDom_passiveSupported;

try
{
	window.addEventListener('t', null, Object.defineProperty({}, 'passive', {
		get: function() { _VirtualDom_passiveSupported = true; }
	}));
}
catch(e) {}



// EVENT HANDLERS


function _VirtualDom_makeCallback(eventNode, initialHandler)
{
	function callback(event)
	{
		var handler = callback.q;
		var result = _Json_runHelp(handler.a, event);

		if (!$elm$core$Result$isOk(result))
		{
			return;
		}

		var tag = $elm$virtual_dom$VirtualDom$toHandlerInt(handler);

		// 0 = Normal
		// 1 = MayStopPropagation
		// 2 = MayPreventDefault
		// 3 = Custom

		var value = result.a;
		var message = !tag ? value : tag < 3 ? value.a : value.message;
		var stopPropagation = tag == 1 ? value.b : tag == 3 && value.stopPropagation;
		var currentEventNode = (
			stopPropagation && event.stopPropagation(),
			(tag == 2 ? value.b : tag == 3 && value.preventDefault) && event.preventDefault(),
			eventNode
		);
		var tagger;
		var i;
		while (tagger = currentEventNode.j)
		{
			if (typeof tagger == 'function')
			{
				message = tagger(message);
			}
			else
			{
				for (var i = tagger.length; i--; )
				{
					message = tagger[i](message);
				}
			}
			currentEventNode = currentEventNode.p;
		}
		currentEventNode(message, stopPropagation); // stopPropagation implies isSync
	}

	callback.q = initialHandler;

	return callback;
}

function _VirtualDom_equalEvents(x, y)
{
	return x.$ == y.$ && _Json_equality(x.a, y.a);
}



// DIFF


// TODO: Should we do patches like in iOS?
//
// type Patch
//   = At Int Patch
//   | Batch (List Patch)
//   | Change ...
//
// How could it not be better?
//
function _VirtualDom_diff(x, y)
{
	var patches = [];
	_VirtualDom_diffHelp(x, y, patches, 0);
	return patches;
}


function _VirtualDom_pushPatch(patches, type, index, data)
{
	var patch = {
		$: type,
		r: index,
		s: data,
		t: undefined,
		u: undefined
	};
	patches.push(patch);
	return patch;
}


function _VirtualDom_diffHelp(x, y, patches, index)
{
	if (x === y)
	{
		return;
	}

	var xType = x.$;
	var yType = y.$;

	// Bail if you run into different types of nodes. Implies that the
	// structure has changed significantly and it's not worth a diff.
	if (xType !== yType)
	{
		if (xType === 1 && yType === 2)
		{
			y = _VirtualDom_dekey(y);
			yType = 1;
		}
		else
		{
			_VirtualDom_pushPatch(patches, 0, index, y);
			return;
		}
	}

	// Now we know that both nodes are the same $.
	switch (yType)
	{
		case 5:
			var xRefs = x.l;
			var yRefs = y.l;
			var i = xRefs.length;
			var same = i === yRefs.length;
			while (same && i--)
			{
				same = xRefs[i] === yRefs[i];
			}
			if (same)
			{
				y.k = x.k;
				return;
			}
			y.k = y.m();
			var subPatches = [];
			_VirtualDom_diffHelp(x.k, y.k, subPatches, 0);
			subPatches.length > 0 && _VirtualDom_pushPatch(patches, 1, index, subPatches);
			return;

		case 4:
			// gather nested taggers
			var xTaggers = x.j;
			var yTaggers = y.j;
			var nesting = false;

			var xSubNode = x.k;
			while (xSubNode.$ === 4)
			{
				nesting = true;

				typeof xTaggers !== 'object'
					? xTaggers = [xTaggers, xSubNode.j]
					: xTaggers.push(xSubNode.j);

				xSubNode = xSubNode.k;
			}

			var ySubNode = y.k;
			while (ySubNode.$ === 4)
			{
				nesting = true;

				typeof yTaggers !== 'object'
					? yTaggers = [yTaggers, ySubNode.j]
					: yTaggers.push(ySubNode.j);

				ySubNode = ySubNode.k;
			}

			// Just bail if different numbers of taggers. This implies the
			// structure of the virtual DOM has changed.
			if (nesting && xTaggers.length !== yTaggers.length)
			{
				_VirtualDom_pushPatch(patches, 0, index, y);
				return;
			}

			// check if taggers are "the same"
			if (nesting ? !_VirtualDom_pairwiseRefEqual(xTaggers, yTaggers) : xTaggers !== yTaggers)
			{
				_VirtualDom_pushPatch(patches, 2, index, yTaggers);
			}

			// diff everything below the taggers
			_VirtualDom_diffHelp(xSubNode, ySubNode, patches, index + 1);
			return;

		case 0:
			if (x.a !== y.a)
			{
				_VirtualDom_pushPatch(patches, 3, index, y.a);
			}
			return;

		case 1:
			_VirtualDom_diffNodes(x, y, patches, index, _VirtualDom_diffKids);
			return;

		case 2:
			_VirtualDom_diffNodes(x, y, patches, index, _VirtualDom_diffKeyedKids);
			return;

		case 3:
			if (x.h !== y.h)
			{
				_VirtualDom_pushPatch(patches, 0, index, y);
				return;
			}

			var factsDiff = _VirtualDom_diffFacts(x.d, y.d);
			factsDiff && _VirtualDom_pushPatch(patches, 4, index, factsDiff);

			var patch = y.i(x.g, y.g);
			patch && _VirtualDom_pushPatch(patches, 5, index, patch);

			return;
	}
}

// assumes the incoming arrays are the same length
function _VirtualDom_pairwiseRefEqual(as, bs)
{
	for (var i = 0; i < as.length; i++)
	{
		if (as[i] !== bs[i])
		{
			return false;
		}
	}

	return true;
}

function _VirtualDom_diffNodes(x, y, patches, index, diffKids)
{
	// Bail if obvious indicators have changed. Implies more serious
	// structural changes such that it's not worth it to diff.
	if (x.c !== y.c || x.f !== y.f)
	{
		_VirtualDom_pushPatch(patches, 0, index, y);
		return;
	}

	var factsDiff = _VirtualDom_diffFacts(x.d, y.d);
	factsDiff && _VirtualDom_pushPatch(patches, 4, index, factsDiff);

	diffKids(x, y, patches, index);
}



// DIFF FACTS


// TODO Instead of creating a new diff object, it's possible to just test if
// there *is* a diff. During the actual patch, do the diff again and make the
// modifications directly. This way, there's no new allocations. Worth it?
function _VirtualDom_diffFacts(x, y, category)
{
	var diff;

	// look for changes and removals
	for (var xKey in x)
	{
		if (xKey === 'a1' || xKey === 'a0' || xKey === 'a3' || xKey === 'a4')
		{
			var subDiff = _VirtualDom_diffFacts(x[xKey], y[xKey] || {}, xKey);
			if (subDiff)
			{
				diff = diff || {};
				diff[xKey] = subDiff;
			}
			continue;
		}

		// remove if not in the new facts
		if (!(xKey in y))
		{
			diff = diff || {};
			diff[xKey] =
				!category
					? (typeof x[xKey] === 'string' ? '' : null)
					:
				(category === 'a1')
					? ''
					:
				(category === 'a0' || category === 'a3')
					? undefined
					:
				{ f: x[xKey].f, o: undefined };

			continue;
		}

		var xValue = x[xKey];
		var yValue = y[xKey];

		// reference equal, so don't worry about it
		if (xValue === yValue && xKey !== 'value' && xKey !== 'checked'
			|| category === 'a0' && _VirtualDom_equalEvents(xValue, yValue))
		{
			continue;
		}

		diff = diff || {};
		diff[xKey] = yValue;
	}

	// add new stuff
	for (var yKey in y)
	{
		if (!(yKey in x))
		{
			diff = diff || {};
			diff[yKey] = y[yKey];
		}
	}

	return diff;
}



// DIFF KIDS


function _VirtualDom_diffKids(xParent, yParent, patches, index)
{
	var xKids = xParent.e;
	var yKids = yParent.e;

	var xLen = xKids.length;
	var yLen = yKids.length;

	// FIGURE OUT IF THERE ARE INSERTS OR REMOVALS

	if (xLen > yLen)
	{
		_VirtualDom_pushPatch(patches, 6, index, {
			v: yLen,
			i: xLen - yLen
		});
	}
	else if (xLen < yLen)
	{
		_VirtualDom_pushPatch(patches, 7, index, {
			v: xLen,
			e: yKids
		});
	}

	// PAIRWISE DIFF EVERYTHING ELSE

	for (var minLen = xLen < yLen ? xLen : yLen, i = 0; i < minLen; i++)
	{
		var xKid = xKids[i];
		_VirtualDom_diffHelp(xKid, yKids[i], patches, ++index);
		index += xKid.b || 0;
	}
}



// KEYED DIFF


function _VirtualDom_diffKeyedKids(xParent, yParent, patches, rootIndex)
{
	var localPatches = [];

	var changes = {}; // Dict String Entry
	var inserts = []; // Array { index : Int, entry : Entry }
	// type Entry = { tag : String, vnode : VNode, index : Int, data : _ }

	var xKids = xParent.e;
	var yKids = yParent.e;
	var xLen = xKids.length;
	var yLen = yKids.length;
	var xIndex = 0;
	var yIndex = 0;

	var index = rootIndex;

	while (xIndex < xLen && yIndex < yLen)
	{
		var x = xKids[xIndex];
		var y = yKids[yIndex];

		var xKey = x.a;
		var yKey = y.a;
		var xNode = x.b;
		var yNode = y.b;

		var newMatch = undefined;
		var oldMatch = undefined;

		// check if keys match

		if (xKey === yKey)
		{
			index++;
			_VirtualDom_diffHelp(xNode, yNode, localPatches, index);
			index += xNode.b || 0;

			xIndex++;
			yIndex++;
			continue;
		}

		// look ahead 1 to detect insertions and removals.

		var xNext = xKids[xIndex + 1];
		var yNext = yKids[yIndex + 1];

		if (xNext)
		{
			var xNextKey = xNext.a;
			var xNextNode = xNext.b;
			oldMatch = yKey === xNextKey;
		}

		if (yNext)
		{
			var yNextKey = yNext.a;
			var yNextNode = yNext.b;
			newMatch = xKey === yNextKey;
		}


		// swap x and y
		if (newMatch && oldMatch)
		{
			index++;
			_VirtualDom_diffHelp(xNode, yNextNode, localPatches, index);
			_VirtualDom_insertNode(changes, localPatches, xKey, yNode, yIndex, inserts);
			index += xNode.b || 0;

			index++;
			_VirtualDom_removeNode(changes, localPatches, xKey, xNextNode, index);
			index += xNextNode.b || 0;

			xIndex += 2;
			yIndex += 2;
			continue;
		}

		// insert y
		if (newMatch)
		{
			index++;
			_VirtualDom_insertNode(changes, localPatches, yKey, yNode, yIndex, inserts);
			_VirtualDom_diffHelp(xNode, yNextNode, localPatches, index);
			index += xNode.b || 0;

			xIndex += 1;
			yIndex += 2;
			continue;
		}

		// remove x
		if (oldMatch)
		{
			index++;
			_VirtualDom_removeNode(changes, localPatches, xKey, xNode, index);
			index += xNode.b || 0;

			index++;
			_VirtualDom_diffHelp(xNextNode, yNode, localPatches, index);
			index += xNextNode.b || 0;

			xIndex += 2;
			yIndex += 1;
			continue;
		}

		// remove x, insert y
		if (xNext && xNextKey === yNextKey)
		{
			index++;
			_VirtualDom_removeNode(changes, localPatches, xKey, xNode, index);
			_VirtualDom_insertNode(changes, localPatches, yKey, yNode, yIndex, inserts);
			index += xNode.b || 0;

			index++;
			_VirtualDom_diffHelp(xNextNode, yNextNode, localPatches, index);
			index += xNextNode.b || 0;

			xIndex += 2;
			yIndex += 2;
			continue;
		}

		break;
	}

	// eat up any remaining nodes with removeNode and insertNode

	while (xIndex < xLen)
	{
		index++;
		var x = xKids[xIndex];
		var xNode = x.b;
		_VirtualDom_removeNode(changes, localPatches, x.a, xNode, index);
		index += xNode.b || 0;
		xIndex++;
	}

	while (yIndex < yLen)
	{
		var endInserts = endInserts || [];
		var y = yKids[yIndex];
		_VirtualDom_insertNode(changes, localPatches, y.a, y.b, undefined, endInserts);
		yIndex++;
	}

	if (localPatches.length > 0 || inserts.length > 0 || endInserts)
	{
		_VirtualDom_pushPatch(patches, 8, rootIndex, {
			w: localPatches,
			x: inserts,
			y: endInserts
		});
	}
}



// CHANGES FROM KEYED DIFF


var _VirtualDom_POSTFIX = '_elmW6BL';


function _VirtualDom_insertNode(changes, localPatches, key, vnode, yIndex, inserts)
{
	var entry = changes[key];

	// never seen this key before
	if (!entry)
	{
		entry = {
			c: 0,
			z: vnode,
			r: yIndex,
			s: undefined
		};

		inserts.push({ r: yIndex, A: entry });
		changes[key] = entry;

		return;
	}

	// this key was removed earlier, a match!
	if (entry.c === 1)
	{
		inserts.push({ r: yIndex, A: entry });

		entry.c = 2;
		var subPatches = [];
		_VirtualDom_diffHelp(entry.z, vnode, subPatches, entry.r);
		entry.r = yIndex;
		entry.s.s = {
			w: subPatches,
			A: entry
		};

		return;
	}

	// this key has already been inserted or moved, a duplicate!
	_VirtualDom_insertNode(changes, localPatches, key + _VirtualDom_POSTFIX, vnode, yIndex, inserts);
}


function _VirtualDom_removeNode(changes, localPatches, key, vnode, index)
{
	var entry = changes[key];

	// never seen this key before
	if (!entry)
	{
		var patch = _VirtualDom_pushPatch(localPatches, 9, index, undefined);

		changes[key] = {
			c: 1,
			z: vnode,
			r: index,
			s: patch
		};

		return;
	}

	// this key was inserted earlier, a match!
	if (entry.c === 0)
	{
		entry.c = 2;
		var subPatches = [];
		_VirtualDom_diffHelp(vnode, entry.z, subPatches, index);

		_VirtualDom_pushPatch(localPatches, 9, index, {
			w: subPatches,
			A: entry
		});

		return;
	}

	// this key has already been removed or moved, a duplicate!
	_VirtualDom_removeNode(changes, localPatches, key + _VirtualDom_POSTFIX, vnode, index);
}



// ADD DOM NODES
//
// Each DOM node has an "index" assigned in order of traversal. It is important
// to minimize our crawl over the actual DOM, so these indexes (along with the
// descendantsCount of virtual nodes) let us skip touching entire subtrees of
// the DOM if we know there are no patches there.


function _VirtualDom_addDomNodes(domNode, vNode, patches, eventNode)
{
	_VirtualDom_addDomNodesHelp(domNode, vNode, patches, 0, 0, vNode.b, eventNode);
}


// assumes `patches` is non-empty and indexes increase monotonically.
function _VirtualDom_addDomNodesHelp(domNode, vNode, patches, i, low, high, eventNode)
{
	var patch = patches[i];
	var index = patch.r;

	while (index === low)
	{
		var patchType = patch.$;

		if (patchType === 1)
		{
			_VirtualDom_addDomNodes(domNode, vNode.k, patch.s, eventNode);
		}
		else if (patchType === 8)
		{
			patch.t = domNode;
			patch.u = eventNode;

			var subPatches = patch.s.w;
			if (subPatches.length > 0)
			{
				_VirtualDom_addDomNodesHelp(domNode, vNode, subPatches, 0, low, high, eventNode);
			}
		}
		else if (patchType === 9)
		{
			patch.t = domNode;
			patch.u = eventNode;

			var data = patch.s;
			if (data)
			{
				data.A.s = domNode;
				var subPatches = data.w;
				if (subPatches.length > 0)
				{
					_VirtualDom_addDomNodesHelp(domNode, vNode, subPatches, 0, low, high, eventNode);
				}
			}
		}
		else
		{
			patch.t = domNode;
			patch.u = eventNode;
		}

		i++;

		if (!(patch = patches[i]) || (index = patch.r) > high)
		{
			return i;
		}
	}

	var tag = vNode.$;

	if (tag === 4)
	{
		var subNode = vNode.k;

		while (subNode.$ === 4)
		{
			subNode = subNode.k;
		}

		return _VirtualDom_addDomNodesHelp(domNode, subNode, patches, i, low + 1, high, domNode.elm_event_node_ref);
	}

	// tag must be 1 or 2 at this point

	var vKids = vNode.e;
	var childNodes = domNode.childNodes;
	for (var j = 0; j < vKids.length; j++)
	{
		low++;
		var vKid = tag === 1 ? vKids[j] : vKids[j].b;
		var nextLow = low + (vKid.b || 0);
		if (low <= index && index <= nextLow)
		{
			i = _VirtualDom_addDomNodesHelp(childNodes[j], vKid, patches, i, low, nextLow, eventNode);
			if (!(patch = patches[i]) || (index = patch.r) > high)
			{
				return i;
			}
		}
		low = nextLow;
	}
	return i;
}



// APPLY PATCHES


function _VirtualDom_applyPatches(rootDomNode, oldVirtualNode, patches, eventNode)
{
	if (patches.length === 0)
	{
		return rootDomNode;
	}

	_VirtualDom_addDomNodes(rootDomNode, oldVirtualNode, patches, eventNode);
	return _VirtualDom_applyPatchesHelp(rootDomNode, patches);
}

function _VirtualDom_applyPatchesHelp(rootDomNode, patches)
{
	for (var i = 0; i < patches.length; i++)
	{
		var patch = patches[i];
		var localDomNode = patch.t
		var newNode = _VirtualDom_applyPatch(localDomNode, patch);
		if (localDomNode === rootDomNode)
		{
			rootDomNode = newNode;
		}
	}
	return rootDomNode;
}

function _VirtualDom_applyPatch(domNode, patch)
{
	switch (patch.$)
	{
		case 0:
			return _VirtualDom_applyPatchRedraw(domNode, patch.s, patch.u);

		case 4:
			_VirtualDom_applyFacts(domNode, patch.u, patch.s);
			return domNode;

		case 3:
			domNode.replaceData(0, domNode.length, patch.s);
			return domNode;

		case 1:
			return _VirtualDom_applyPatchesHelp(domNode, patch.s);

		case 2:
			if (domNode.elm_event_node_ref)
			{
				domNode.elm_event_node_ref.j = patch.s;
			}
			else
			{
				domNode.elm_event_node_ref = { j: patch.s, p: patch.u };
			}
			return domNode;

		case 6:
			var data = patch.s;
			for (var i = 0; i < data.i; i++)
			{
				domNode.removeChild(domNode.childNodes[data.v]);
			}
			return domNode;

		case 7:
			var data = patch.s;
			var kids = data.e;
			var i = data.v;
			var theEnd = domNode.childNodes[i];
			for (; i < kids.length; i++)
			{
				domNode.insertBefore(_VirtualDom_render(kids[i], patch.u), theEnd);
			}
			return domNode;

		case 9:
			var data = patch.s;
			if (!data)
			{
				domNode.parentNode.removeChild(domNode);
				return domNode;
			}
			var entry = data.A;
			if (typeof entry.r !== 'undefined')
			{
				domNode.parentNode.removeChild(domNode);
			}
			entry.s = _VirtualDom_applyPatchesHelp(domNode, data.w);
			return domNode;

		case 8:
			return _VirtualDom_applyPatchReorder(domNode, patch);

		case 5:
			return patch.s(domNode);

		default:
			_Debug_crash(10); // 'Ran into an unknown patch!'
	}
}


function _VirtualDom_applyPatchRedraw(domNode, vNode, eventNode)
{
	var parentNode = domNode.parentNode;
	var newNode = _VirtualDom_render(vNode, eventNode);

	if (!newNode.elm_event_node_ref)
	{
		newNode.elm_event_node_ref = domNode.elm_event_node_ref;
	}

	if (parentNode && newNode !== domNode)
	{
		parentNode.replaceChild(newNode, domNode);
	}
	return newNode;
}


function _VirtualDom_applyPatchReorder(domNode, patch)
{
	var data = patch.s;

	// remove end inserts
	var frag = _VirtualDom_applyPatchReorderEndInsertsHelp(data.y, patch);

	// removals
	domNode = _VirtualDom_applyPatchesHelp(domNode, data.w);

	// inserts
	var inserts = data.x;
	for (var i = 0; i < inserts.length; i++)
	{
		var insert = inserts[i];
		var entry = insert.A;
		var node = entry.c === 2
			? entry.s
			: _VirtualDom_render(entry.z, patch.u);
		domNode.insertBefore(node, domNode.childNodes[insert.r]);
	}

	// add end inserts
	if (frag)
	{
		_VirtualDom_appendChild(domNode, frag);
	}

	return domNode;
}


function _VirtualDom_applyPatchReorderEndInsertsHelp(endInserts, patch)
{
	if (!endInserts)
	{
		return;
	}

	var frag = _VirtualDom_doc.createDocumentFragment();
	for (var i = 0; i < endInserts.length; i++)
	{
		var insert = endInserts[i];
		var entry = insert.A;
		_VirtualDom_appendChild(frag, entry.c === 2
			? entry.s
			: _VirtualDom_render(entry.z, patch.u)
		);
	}
	return frag;
}


function _VirtualDom_virtualize(node)
{
	// TEXT NODES

	if (node.nodeType === 3)
	{
		return _VirtualDom_text(node.textContent);
	}


	// WEIRD NODES

	if (node.nodeType !== 1)
	{
		return _VirtualDom_text('');
	}


	// ELEMENT NODES

	var attrList = _List_Nil;
	var attrs = node.attributes;
	for (var i = attrs.length; i--; )
	{
		var attr = attrs[i];
		var name = attr.name;
		var value = attr.value;
		attrList = _List_Cons( A2(_VirtualDom_attribute, name, value), attrList );
	}

	var tag = node.tagName.toLowerCase();
	var kidList = _List_Nil;
	var kids = node.childNodes;

	for (var i = kids.length; i--; )
	{
		kidList = _List_Cons(_VirtualDom_virtualize(kids[i]), kidList);
	}
	return A3(_VirtualDom_node, tag, attrList, kidList);
}

function _VirtualDom_dekey(keyedNode)
{
	var keyedKids = keyedNode.e;
	var len = keyedKids.length;
	var kids = new Array(len);
	for (var i = 0; i < len; i++)
	{
		kids[i] = keyedKids[i].b;
	}

	return {
		$: 1,
		c: keyedNode.c,
		d: keyedNode.d,
		e: kids,
		f: keyedNode.f,
		b: keyedNode.b
	};
}




// ELEMENT


var _Debugger_element;

var _Browser_element = _Debugger_element || F4(function(impl, flagDecoder, debugMetadata, args)
{
	return _Platform_initialize(
		flagDecoder,
		args,
		impl.init,
		impl.update,
		impl.subscriptions,
		function(sendToApp, initialModel) {
			var view = impl.view;
			/**_UNUSED/
			var domNode = args['node'];
			//*/
			/**/
			var domNode = args && args['node'] ? args['node'] : _Debug_crash(0);
			//*/
			var currNode = _VirtualDom_virtualize(domNode);

			return _Browser_makeAnimator(initialModel, function(model)
			{
				var nextNode = view(model);
				var patches = _VirtualDom_diff(currNode, nextNode);
				domNode = _VirtualDom_applyPatches(domNode, currNode, patches, sendToApp);
				currNode = nextNode;
			});
		}
	);
});



// DOCUMENT


var _Debugger_document;

var _Browser_document = _Debugger_document || F4(function(impl, flagDecoder, debugMetadata, args)
{
	return _Platform_initialize(
		flagDecoder,
		args,
		impl.init,
		impl.update,
		impl.subscriptions,
		function(sendToApp, initialModel) {
			var divertHrefToApp = impl.setup && impl.setup(sendToApp)
			var view = impl.view;
			var title = _VirtualDom_doc.title;
			var bodyNode = _VirtualDom_doc.body;
			var currNode = _VirtualDom_virtualize(bodyNode);
			return _Browser_makeAnimator(initialModel, function(model)
			{
				_VirtualDom_divertHrefToApp = divertHrefToApp;
				var doc = view(model);
				var nextNode = _VirtualDom_node('body')(_List_Nil)(doc.body);
				var patches = _VirtualDom_diff(currNode, nextNode);
				bodyNode = _VirtualDom_applyPatches(bodyNode, currNode, patches, sendToApp);
				currNode = nextNode;
				_VirtualDom_divertHrefToApp = 0;
				(title !== doc.title) && (_VirtualDom_doc.title = title = doc.title);
			});
		}
	);
});



// ANIMATION


var _Browser_cancelAnimationFrame =
	typeof cancelAnimationFrame !== 'undefined'
		? cancelAnimationFrame
		: function(id) { clearTimeout(id); };

var _Browser_requestAnimationFrame =
	typeof requestAnimationFrame !== 'undefined'
		? requestAnimationFrame
		: function(callback) { return setTimeout(callback, 1000 / 60); };


function _Browser_makeAnimator(model, draw)
{
	draw(model);

	var state = 0;

	function updateIfNeeded()
	{
		state = state === 1
			? 0
			: ( _Browser_requestAnimationFrame(updateIfNeeded), draw(model), 1 );
	}

	return function(nextModel, isSync)
	{
		model = nextModel;

		isSync
			? ( draw(model),
				state === 2 && (state = 1)
				)
			: ( state === 0 && _Browser_requestAnimationFrame(updateIfNeeded),
				state = 2
				);
	};
}



// APPLICATION


function _Browser_application(impl)
{
	var onUrlChange = impl.onUrlChange;
	var onUrlRequest = impl.onUrlRequest;
	var key = function() { key.a(onUrlChange(_Browser_getUrl())); };

	return _Browser_document({
		setup: function(sendToApp)
		{
			key.a = sendToApp;
			_Browser_window.addEventListener('popstate', key);
			_Browser_window.navigator.userAgent.indexOf('Trident') < 0 || _Browser_window.addEventListener('hashchange', key);

			return F2(function(domNode, event)
			{
				if (!event.ctrlKey && !event.metaKey && !event.shiftKey && event.button < 1 && !domNode.target && !domNode.hasAttribute('download'))
				{
					event.preventDefault();
					var href = domNode.href;
					var curr = _Browser_getUrl();
					var next = $elm$url$Url$fromString(href).a;
					sendToApp(onUrlRequest(
						(next
							&& curr.protocol === next.protocol
							&& curr.host === next.host
							&& curr.port_.a === next.port_.a
						)
							? $elm$browser$Browser$Internal(next)
							: $elm$browser$Browser$External(href)
					));
				}
			});
		},
		init: function(flags)
		{
			return A3(impl.init, flags, _Browser_getUrl(), key);
		},
		view: impl.view,
		update: impl.update,
		subscriptions: impl.subscriptions
	});
}

function _Browser_getUrl()
{
	return $elm$url$Url$fromString(_VirtualDom_doc.location.href).a || _Debug_crash(1);
}

var _Browser_go = F2(function(key, n)
{
	return A2($elm$core$Task$perform, $elm$core$Basics$never, _Scheduler_binding(function() {
		n && history.go(n);
		key();
	}));
});

var _Browser_pushUrl = F2(function(key, url)
{
	return A2($elm$core$Task$perform, $elm$core$Basics$never, _Scheduler_binding(function() {
		history.pushState({}, '', url);
		key();
	}));
});

var _Browser_replaceUrl = F2(function(key, url)
{
	return A2($elm$core$Task$perform, $elm$core$Basics$never, _Scheduler_binding(function() {
		history.replaceState({}, '', url);
		key();
	}));
});



// GLOBAL EVENTS


var _Browser_fakeNode = { addEventListener: function() {}, removeEventListener: function() {} };
var _Browser_doc = typeof document !== 'undefined' ? document : _Browser_fakeNode;
var _Browser_window = typeof window !== 'undefined' ? window : _Browser_fakeNode;

var _Browser_on = F3(function(node, eventName, sendToSelf)
{
	return _Scheduler_spawn(_Scheduler_binding(function(callback)
	{
		function handler(event)	{ _Scheduler_rawSpawn(sendToSelf(event)); }
		node.addEventListener(eventName, handler, _VirtualDom_passiveSupported && { passive: true });
		return function() { node.removeEventListener(eventName, handler); };
	}));
});

var _Browser_decodeEvent = F2(function(decoder, event)
{
	var result = _Json_runHelp(decoder, event);
	return $elm$core$Result$isOk(result) ? $elm$core$Maybe$Just(result.a) : $elm$core$Maybe$Nothing;
});



// PAGE VISIBILITY


function _Browser_visibilityInfo()
{
	return (typeof _VirtualDom_doc.hidden !== 'undefined')
		? { hidden: 'hidden', change: 'visibilitychange' }
		:
	(typeof _VirtualDom_doc.mozHidden !== 'undefined')
		? { hidden: 'mozHidden', change: 'mozvisibilitychange' }
		:
	(typeof _VirtualDom_doc.msHidden !== 'undefined')
		? { hidden: 'msHidden', change: 'msvisibilitychange' }
		:
	(typeof _VirtualDom_doc.webkitHidden !== 'undefined')
		? { hidden: 'webkitHidden', change: 'webkitvisibilitychange' }
		: { hidden: 'hidden', change: 'visibilitychange' };
}



// ANIMATION FRAMES


function _Browser_rAF()
{
	return _Scheduler_binding(function(callback)
	{
		var id = _Browser_requestAnimationFrame(function() {
			callback(_Scheduler_succeed(Date.now()));
		});

		return function() {
			_Browser_cancelAnimationFrame(id);
		};
	});
}


function _Browser_now()
{
	return _Scheduler_binding(function(callback)
	{
		callback(_Scheduler_succeed(Date.now()));
	});
}



// DOM STUFF


function _Browser_withNode(id, doStuff)
{
	return _Scheduler_binding(function(callback)
	{
		_Browser_requestAnimationFrame(function() {
			var node = document.getElementById(id);
			callback(node
				? _Scheduler_succeed(doStuff(node))
				: _Scheduler_fail($elm$browser$Browser$Dom$NotFound(id))
			);
		});
	});
}


function _Browser_withWindow(doStuff)
{
	return _Scheduler_binding(function(callback)
	{
		_Browser_requestAnimationFrame(function() {
			callback(_Scheduler_succeed(doStuff()));
		});
	});
}


// FOCUS and BLUR


var _Browser_call = F2(function(functionName, id)
{
	return _Browser_withNode(id, function(node) {
		node[functionName]();
		return _Utils_Tuple0;
	});
});



// WINDOW VIEWPORT


function _Browser_getViewport()
{
	return {
		scene: _Browser_getScene(),
		viewport: {
			x: _Browser_window.pageXOffset,
			y: _Browser_window.pageYOffset,
			width: _Browser_doc.documentElement.clientWidth,
			height: _Browser_doc.documentElement.clientHeight
		}
	};
}

function _Browser_getScene()
{
	var body = _Browser_doc.body;
	var elem = _Browser_doc.documentElement;
	return {
		width: Math.max(body.scrollWidth, body.offsetWidth, elem.scrollWidth, elem.offsetWidth, elem.clientWidth),
		height: Math.max(body.scrollHeight, body.offsetHeight, elem.scrollHeight, elem.offsetHeight, elem.clientHeight)
	};
}

var _Browser_setViewport = F2(function(x, y)
{
	return _Browser_withWindow(function()
	{
		_Browser_window.scroll(x, y);
		return _Utils_Tuple0;
	});
});



// ELEMENT VIEWPORT


function _Browser_getViewportOf(id)
{
	return _Browser_withNode(id, function(node)
	{
		return {
			scene: {
				width: node.scrollWidth,
				height: node.scrollHeight
			},
			viewport: {
				x: node.scrollLeft,
				y: node.scrollTop,
				width: node.clientWidth,
				height: node.clientHeight
			}
		};
	});
}


var _Browser_setViewportOf = F3(function(id, x, y)
{
	return _Browser_withNode(id, function(node)
	{
		node.scrollLeft = x;
		node.scrollTop = y;
		return _Utils_Tuple0;
	});
});



// ELEMENT


function _Browser_getElement(id)
{
	return _Browser_withNode(id, function(node)
	{
		var rect = node.getBoundingClientRect();
		var x = _Browser_window.pageXOffset;
		var y = _Browser_window.pageYOffset;
		return {
			scene: _Browser_getScene(),
			viewport: {
				x: x,
				y: y,
				width: _Browser_doc.documentElement.clientWidth,
				height: _Browser_doc.documentElement.clientHeight
			},
			element: {
				x: x + rect.left,
				y: y + rect.top,
				width: rect.width,
				height: rect.height
			}
		};
	});
}



// LOAD and RELOAD


function _Browser_reload(skipCache)
{
	return A2($elm$core$Task$perform, $elm$core$Basics$never, _Scheduler_binding(function(callback)
	{
		_VirtualDom_doc.location.reload(skipCache);
	}));
}

function _Browser_load(url)
{
	return A2($elm$core$Task$perform, $elm$core$Basics$never, _Scheduler_binding(function(callback)
	{
		try
		{
			_Browser_window.location = url;
		}
		catch(err)
		{
			// Only Firefox can throw a NS_ERROR_MALFORMED_URI exception here.
			// Other browsers reload the page, so let's be consistent about that.
			_VirtualDom_doc.location.reload(false);
		}
	}));
}



// SEND REQUEST

var _Http_toTask = F3(function(router, toTask, request)
{
	return _Scheduler_binding(function(callback)
	{
		function done(response) {
			callback(toTask(request.expect.a(response)));
		}

		var xhr = new XMLHttpRequest();
		xhr.addEventListener('error', function() { done($elm$http$Http$NetworkError_); });
		xhr.addEventListener('timeout', function() { done($elm$http$Http$Timeout_); });
		xhr.addEventListener('load', function() { done(_Http_toResponse(request.expect.b, xhr)); });
		$elm$core$Maybe$isJust(request.tracker) && _Http_track(router, xhr, request.tracker.a);

		try {
			xhr.open(request.method, request.url, true);
		} catch (e) {
			return done($elm$http$Http$BadUrl_(request.url));
		}

		_Http_configureRequest(xhr, request);

		request.body.a && xhr.setRequestHeader('Content-Type', request.body.a);
		xhr.send(request.body.b);

		return function() { xhr.c = true; xhr.abort(); };
	});
});


// CONFIGURE

function _Http_configureRequest(xhr, request)
{
	for (var headers = request.headers; headers.b; headers = headers.b) // WHILE_CONS
	{
		xhr.setRequestHeader(headers.a.a, headers.a.b);
	}
	xhr.timeout = request.timeout.a || 0;
	xhr.responseType = request.expect.d;
	xhr.withCredentials = request.allowCookiesFromOtherDomains;
}


// RESPONSES

function _Http_toResponse(toBody, xhr)
{
	return A2(
		200 <= xhr.status && xhr.status < 300 ? $elm$http$Http$GoodStatus_ : $elm$http$Http$BadStatus_,
		_Http_toMetadata(xhr),
		toBody(xhr.response)
	);
}


// METADATA

function _Http_toMetadata(xhr)
{
	return {
		url: xhr.responseURL,
		statusCode: xhr.status,
		statusText: xhr.statusText,
		headers: _Http_parseHeaders(xhr.getAllResponseHeaders())
	};
}


// HEADERS

function _Http_parseHeaders(rawHeaders)
{
	if (!rawHeaders)
	{
		return $elm$core$Dict$empty;
	}

	var headers = $elm$core$Dict$empty;
	var headerPairs = rawHeaders.split('\r\n');
	for (var i = headerPairs.length; i--; )
	{
		var headerPair = headerPairs[i];
		var index = headerPair.indexOf(': ');
		if (index > 0)
		{
			var key = headerPair.substring(0, index);
			var value = headerPair.substring(index + 2);

			headers = A3($elm$core$Dict$update, key, function(oldValue) {
				return $elm$core$Maybe$Just($elm$core$Maybe$isJust(oldValue)
					? value + ', ' + oldValue.a
					: value
				);
			}, headers);
		}
	}
	return headers;
}


// EXPECT

var _Http_expect = F3(function(type, toBody, toValue)
{
	return {
		$: 0,
		d: type,
		b: toBody,
		a: toValue
	};
});

var _Http_mapExpect = F2(function(func, expect)
{
	return {
		$: 0,
		d: expect.d,
		b: expect.b,
		a: function(x) { return func(expect.a(x)); }
	};
});

function _Http_toDataView(arrayBuffer)
{
	return new DataView(arrayBuffer);
}


// BODY and PARTS

var _Http_emptyBody = { $: 0 };
var _Http_pair = F2(function(a, b) { return { $: 0, a: a, b: b }; });

function _Http_toFormData(parts)
{
	for (var formData = new FormData(); parts.b; parts = parts.b) // WHILE_CONS
	{
		var part = parts.a;
		formData.append(part.a, part.b);
	}
	return formData;
}

var _Http_bytesToBlob = F2(function(mime, bytes)
{
	return new Blob([bytes], { type: mime });
});


// PROGRESS

function _Http_track(router, xhr, tracker)
{
	// TODO check out lengthComputable on loadstart event

	xhr.upload.addEventListener('progress', function(event) {
		if (xhr.c) { return; }
		_Scheduler_rawSpawn(A2($elm$core$Platform$sendToSelf, router, _Utils_Tuple2(tracker, $elm$http$Http$Sending({
			sent: event.loaded,
			size: event.total
		}))));
	});
	xhr.addEventListener('progress', function(event) {
		if (xhr.c) { return; }
		_Scheduler_rawSpawn(A2($elm$core$Platform$sendToSelf, router, _Utils_Tuple2(tracker, $elm$http$Http$Receiving({
			received: event.loaded,
			size: event.lengthComputable ? $elm$core$Maybe$Just(event.total) : $elm$core$Maybe$Nothing
		}))));
	});
}

// CREATE

var _Regex_never = /.^/;

var _Regex_fromStringWith = F2(function(options, string)
{
	var flags = 'g';
	if (options.multiline) { flags += 'm'; }
	if (options.caseInsensitive) { flags += 'i'; }

	try
	{
		return $elm$core$Maybe$Just(new RegExp(string, flags));
	}
	catch(error)
	{
		return $elm$core$Maybe$Nothing;
	}
});


// USE

var _Regex_contains = F2(function(re, string)
{
	return string.match(re) !== null;
});


var _Regex_findAtMost = F3(function(n, re, str)
{
	var out = [];
	var number = 0;
	var string = str;
	var lastIndex = re.lastIndex;
	var prevLastIndex = -1;
	var result;
	while (number++ < n && (result = re.exec(string)))
	{
		if (prevLastIndex == re.lastIndex) break;
		var i = result.length - 1;
		var subs = new Array(i);
		while (i > 0)
		{
			var submatch = result[i];
			subs[--i] = submatch
				? $elm$core$Maybe$Just(submatch)
				: $elm$core$Maybe$Nothing;
		}
		out.push(A4($elm$regex$Regex$Match, result[0], result.index, number, _List_fromArray(subs)));
		prevLastIndex = re.lastIndex;
	}
	re.lastIndex = lastIndex;
	return _List_fromArray(out);
});


var _Regex_replaceAtMost = F4(function(n, re, replacer, string)
{
	var count = 0;
	function jsReplacer(match)
	{
		if (count++ >= n)
		{
			return match;
		}
		var i = arguments.length - 3;
		var submatches = new Array(i);
		while (i > 0)
		{
			var submatch = arguments[i];
			submatches[--i] = submatch
				? $elm$core$Maybe$Just(submatch)
				: $elm$core$Maybe$Nothing;
		}
		return replacer(A4($elm$regex$Regex$Match, match, arguments[arguments.length - 2], count, _List_fromArray(submatches)));
	}
	return string.replace(re, jsReplacer);
});

var _Regex_splitAtMost = F3(function(n, re, str)
{
	var string = str;
	var out = [];
	var start = re.lastIndex;
	var restoreLastIndex = re.lastIndex;
	while (n--)
	{
		var result = re.exec(string);
		if (!result) break;
		out.push(string.slice(start, result.index));
		start = re.lastIndex;
	}
	out.push(string.slice(start));
	re.lastIndex = restoreLastIndex;
	return _List_fromArray(out);
});

var _Regex_infinity = Infinity;
var $elm$core$Basics$EQ = {$: 'EQ'};
var $elm$core$Basics$GT = {$: 'GT'};
var $elm$core$Basics$LT = {$: 'LT'};
var $elm$core$List$cons = _List_cons;
var $elm$core$Dict$foldr = F3(
	function (func, acc, t) {
		foldr:
		while (true) {
			if (t.$ === 'RBEmpty_elm_builtin') {
				return acc;
			} else {
				var key = t.b;
				var value = t.c;
				var left = t.d;
				var right = t.e;
				var $temp$func = func,
					$temp$acc = A3(
					func,
					key,
					value,
					A3($elm$core$Dict$foldr, func, acc, right)),
					$temp$t = left;
				func = $temp$func;
				acc = $temp$acc;
				t = $temp$t;
				continue foldr;
			}
		}
	});
var $elm$core$Dict$toList = function (dict) {
	return A3(
		$elm$core$Dict$foldr,
		F3(
			function (key, value, list) {
				return A2(
					$elm$core$List$cons,
					_Utils_Tuple2(key, value),
					list);
			}),
		_List_Nil,
		dict);
};
var $elm$core$Dict$keys = function (dict) {
	return A3(
		$elm$core$Dict$foldr,
		F3(
			function (key, value, keyList) {
				return A2($elm$core$List$cons, key, keyList);
			}),
		_List_Nil,
		dict);
};
var $elm$core$Set$toList = function (_v0) {
	var dict = _v0.a;
	return $elm$core$Dict$keys(dict);
};
var $elm$core$Elm$JsArray$foldr = _JsArray_foldr;
var $elm$core$Array$foldr = F3(
	function (func, baseCase, _v0) {
		var tree = _v0.c;
		var tail = _v0.d;
		var helper = F2(
			function (node, acc) {
				if (node.$ === 'SubTree') {
					var subTree = node.a;
					return A3($elm$core$Elm$JsArray$foldr, helper, acc, subTree);
				} else {
					var values = node.a;
					return A3($elm$core$Elm$JsArray$foldr, func, acc, values);
				}
			});
		return A3(
			$elm$core$Elm$JsArray$foldr,
			helper,
			A3($elm$core$Elm$JsArray$foldr, func, baseCase, tail),
			tree);
	});
var $elm$core$Array$toList = function (array) {
	return A3($elm$core$Array$foldr, $elm$core$List$cons, _List_Nil, array);
};
var $elm$core$Result$Err = function (a) {
	return {$: 'Err', a: a};
};
var $elm$json$Json$Decode$Failure = F2(
	function (a, b) {
		return {$: 'Failure', a: a, b: b};
	});
var $elm$json$Json$Decode$Field = F2(
	function (a, b) {
		return {$: 'Field', a: a, b: b};
	});
var $elm$json$Json$Decode$Index = F2(
	function (a, b) {
		return {$: 'Index', a: a, b: b};
	});
var $elm$core$Result$Ok = function (a) {
	return {$: 'Ok', a: a};
};
var $elm$json$Json$Decode$OneOf = function (a) {
	return {$: 'OneOf', a: a};
};
var $elm$core$Basics$False = {$: 'False'};
var $elm$core$Basics$add = _Basics_add;
var $elm$core$Maybe$Just = function (a) {
	return {$: 'Just', a: a};
};
var $elm$core$Maybe$Nothing = {$: 'Nothing'};
var $elm$core$String$all = _String_all;
var $elm$core$Basics$and = _Basics_and;
var $elm$core$Basics$append = _Utils_append;
var $elm$json$Json$Encode$encode = _Json_encode;
var $elm$core$String$fromInt = _String_fromNumber;
var $elm$core$String$join = F2(
	function (sep, chunks) {
		return A2(
			_String_join,
			sep,
			_List_toArray(chunks));
	});
var $elm$core$String$split = F2(
	function (sep, string) {
		return _List_fromArray(
			A2(_String_split, sep, string));
	});
var $elm$json$Json$Decode$indent = function (str) {
	return A2(
		$elm$core$String$join,
		'\n    ',
		A2($elm$core$String$split, '\n', str));
};
var $elm$core$List$foldl = F3(
	function (func, acc, list) {
		foldl:
		while (true) {
			if (!list.b) {
				return acc;
			} else {
				var x = list.a;
				var xs = list.b;
				var $temp$func = func,
					$temp$acc = A2(func, x, acc),
					$temp$list = xs;
				func = $temp$func;
				acc = $temp$acc;
				list = $temp$list;
				continue foldl;
			}
		}
	});
var $elm$core$List$length = function (xs) {
	return A3(
		$elm$core$List$foldl,
		F2(
			function (_v0, i) {
				return i + 1;
			}),
		0,
		xs);
};
var $elm$core$List$map2 = _List_map2;
var $elm$core$Basics$le = _Utils_le;
var $elm$core$Basics$sub = _Basics_sub;
var $elm$core$List$rangeHelp = F3(
	function (lo, hi, list) {
		rangeHelp:
		while (true) {
			if (_Utils_cmp(lo, hi) < 1) {
				var $temp$lo = lo,
					$temp$hi = hi - 1,
					$temp$list = A2($elm$core$List$cons, hi, list);
				lo = $temp$lo;
				hi = $temp$hi;
				list = $temp$list;
				continue rangeHelp;
			} else {
				return list;
			}
		}
	});
var $elm$core$List$range = F2(
	function (lo, hi) {
		return A3($elm$core$List$rangeHelp, lo, hi, _List_Nil);
	});
var $elm$core$List$indexedMap = F2(
	function (f, xs) {
		return A3(
			$elm$core$List$map2,
			f,
			A2(
				$elm$core$List$range,
				0,
				$elm$core$List$length(xs) - 1),
			xs);
	});
var $elm$core$Char$toCode = _Char_toCode;
var $elm$core$Char$isLower = function (_char) {
	var code = $elm$core$Char$toCode(_char);
	return (97 <= code) && (code <= 122);
};
var $elm$core$Char$isUpper = function (_char) {
	var code = $elm$core$Char$toCode(_char);
	return (code <= 90) && (65 <= code);
};
var $elm$core$Basics$or = _Basics_or;
var $elm$core$Char$isAlpha = function (_char) {
	return $elm$core$Char$isLower(_char) || $elm$core$Char$isUpper(_char);
};
var $elm$core$Char$isDigit = function (_char) {
	var code = $elm$core$Char$toCode(_char);
	return (code <= 57) && (48 <= code);
};
var $elm$core$Char$isAlphaNum = function (_char) {
	return $elm$core$Char$isLower(_char) || ($elm$core$Char$isUpper(_char) || $elm$core$Char$isDigit(_char));
};
var $elm$core$List$reverse = function (list) {
	return A3($elm$core$List$foldl, $elm$core$List$cons, _List_Nil, list);
};
var $elm$core$String$uncons = _String_uncons;
var $elm$json$Json$Decode$errorOneOf = F2(
	function (i, error) {
		return '\n\n(' + ($elm$core$String$fromInt(i + 1) + (') ' + $elm$json$Json$Decode$indent(
			$elm$json$Json$Decode$errorToString(error))));
	});
var $elm$json$Json$Decode$errorToString = function (error) {
	return A2($elm$json$Json$Decode$errorToStringHelp, error, _List_Nil);
};
var $elm$json$Json$Decode$errorToStringHelp = F2(
	function (error, context) {
		errorToStringHelp:
		while (true) {
			switch (error.$) {
				case 'Field':
					var f = error.a;
					var err = error.b;
					var isSimple = function () {
						var _v1 = $elm$core$String$uncons(f);
						if (_v1.$ === 'Nothing') {
							return false;
						} else {
							var _v2 = _v1.a;
							var _char = _v2.a;
							var rest = _v2.b;
							return $elm$core$Char$isAlpha(_char) && A2($elm$core$String$all, $elm$core$Char$isAlphaNum, rest);
						}
					}();
					var fieldName = isSimple ? ('.' + f) : ('[\'' + (f + '\']'));
					var $temp$error = err,
						$temp$context = A2($elm$core$List$cons, fieldName, context);
					error = $temp$error;
					context = $temp$context;
					continue errorToStringHelp;
				case 'Index':
					var i = error.a;
					var err = error.b;
					var indexName = '[' + ($elm$core$String$fromInt(i) + ']');
					var $temp$error = err,
						$temp$context = A2($elm$core$List$cons, indexName, context);
					error = $temp$error;
					context = $temp$context;
					continue errorToStringHelp;
				case 'OneOf':
					var errors = error.a;
					if (!errors.b) {
						return 'Ran into a Json.Decode.oneOf with no possibilities' + function () {
							if (!context.b) {
								return '!';
							} else {
								return ' at json' + A2(
									$elm$core$String$join,
									'',
									$elm$core$List$reverse(context));
							}
						}();
					} else {
						if (!errors.b.b) {
							var err = errors.a;
							var $temp$error = err,
								$temp$context = context;
							error = $temp$error;
							context = $temp$context;
							continue errorToStringHelp;
						} else {
							var starter = function () {
								if (!context.b) {
									return 'Json.Decode.oneOf';
								} else {
									return 'The Json.Decode.oneOf at json' + A2(
										$elm$core$String$join,
										'',
										$elm$core$List$reverse(context));
								}
							}();
							var introduction = starter + (' failed in the following ' + ($elm$core$String$fromInt(
								$elm$core$List$length(errors)) + ' ways:'));
							return A2(
								$elm$core$String$join,
								'\n\n',
								A2(
									$elm$core$List$cons,
									introduction,
									A2($elm$core$List$indexedMap, $elm$json$Json$Decode$errorOneOf, errors)));
						}
					}
				default:
					var msg = error.a;
					var json = error.b;
					var introduction = function () {
						if (!context.b) {
							return 'Problem with the given value:\n\n';
						} else {
							return 'Problem with the value at json' + (A2(
								$elm$core$String$join,
								'',
								$elm$core$List$reverse(context)) + ':\n\n    ');
						}
					}();
					return introduction + ($elm$json$Json$Decode$indent(
						A2($elm$json$Json$Encode$encode, 4, json)) + ('\n\n' + msg));
			}
		}
	});
var $elm$core$Array$branchFactor = 32;
var $elm$core$Array$Array_elm_builtin = F4(
	function (a, b, c, d) {
		return {$: 'Array_elm_builtin', a: a, b: b, c: c, d: d};
	});
var $elm$core$Elm$JsArray$empty = _JsArray_empty;
var $elm$core$Basics$ceiling = _Basics_ceiling;
var $elm$core$Basics$fdiv = _Basics_fdiv;
var $elm$core$Basics$logBase = F2(
	function (base, number) {
		return _Basics_log(number) / _Basics_log(base);
	});
var $elm$core$Basics$toFloat = _Basics_toFloat;
var $elm$core$Array$shiftStep = $elm$core$Basics$ceiling(
	A2($elm$core$Basics$logBase, 2, $elm$core$Array$branchFactor));
var $elm$core$Array$empty = A4($elm$core$Array$Array_elm_builtin, 0, $elm$core$Array$shiftStep, $elm$core$Elm$JsArray$empty, $elm$core$Elm$JsArray$empty);
var $elm$core$Elm$JsArray$initialize = _JsArray_initialize;
var $elm$core$Array$Leaf = function (a) {
	return {$: 'Leaf', a: a};
};
var $elm$core$Basics$apL = F2(
	function (f, x) {
		return f(x);
	});
var $elm$core$Basics$apR = F2(
	function (x, f) {
		return f(x);
	});
var $elm$core$Basics$eq = _Utils_equal;
var $elm$core$Basics$floor = _Basics_floor;
var $elm$core$Elm$JsArray$length = _JsArray_length;
var $elm$core$Basics$gt = _Utils_gt;
var $elm$core$Basics$max = F2(
	function (x, y) {
		return (_Utils_cmp(x, y) > 0) ? x : y;
	});
var $elm$core$Basics$mul = _Basics_mul;
var $elm$core$Array$SubTree = function (a) {
	return {$: 'SubTree', a: a};
};
var $elm$core$Elm$JsArray$initializeFromList = _JsArray_initializeFromList;
var $elm$core$Array$compressNodes = F2(
	function (nodes, acc) {
		compressNodes:
		while (true) {
			var _v0 = A2($elm$core$Elm$JsArray$initializeFromList, $elm$core$Array$branchFactor, nodes);
			var node = _v0.a;
			var remainingNodes = _v0.b;
			var newAcc = A2(
				$elm$core$List$cons,
				$elm$core$Array$SubTree(node),
				acc);
			if (!remainingNodes.b) {
				return $elm$core$List$reverse(newAcc);
			} else {
				var $temp$nodes = remainingNodes,
					$temp$acc = newAcc;
				nodes = $temp$nodes;
				acc = $temp$acc;
				continue compressNodes;
			}
		}
	});
var $elm$core$Tuple$first = function (_v0) {
	var x = _v0.a;
	return x;
};
var $elm$core$Array$treeFromBuilder = F2(
	function (nodeList, nodeListSize) {
		treeFromBuilder:
		while (true) {
			var newNodeSize = $elm$core$Basics$ceiling(nodeListSize / $elm$core$Array$branchFactor);
			if (newNodeSize === 1) {
				return A2($elm$core$Elm$JsArray$initializeFromList, $elm$core$Array$branchFactor, nodeList).a;
			} else {
				var $temp$nodeList = A2($elm$core$Array$compressNodes, nodeList, _List_Nil),
					$temp$nodeListSize = newNodeSize;
				nodeList = $temp$nodeList;
				nodeListSize = $temp$nodeListSize;
				continue treeFromBuilder;
			}
		}
	});
var $elm$core$Array$builderToArray = F2(
	function (reverseNodeList, builder) {
		if (!builder.nodeListSize) {
			return A4(
				$elm$core$Array$Array_elm_builtin,
				$elm$core$Elm$JsArray$length(builder.tail),
				$elm$core$Array$shiftStep,
				$elm$core$Elm$JsArray$empty,
				builder.tail);
		} else {
			var treeLen = builder.nodeListSize * $elm$core$Array$branchFactor;
			var depth = $elm$core$Basics$floor(
				A2($elm$core$Basics$logBase, $elm$core$Array$branchFactor, treeLen - 1));
			var correctNodeList = reverseNodeList ? $elm$core$List$reverse(builder.nodeList) : builder.nodeList;
			var tree = A2($elm$core$Array$treeFromBuilder, correctNodeList, builder.nodeListSize);
			return A4(
				$elm$core$Array$Array_elm_builtin,
				$elm$core$Elm$JsArray$length(builder.tail) + treeLen,
				A2($elm$core$Basics$max, 5, depth * $elm$core$Array$shiftStep),
				tree,
				builder.tail);
		}
	});
var $elm$core$Basics$idiv = _Basics_idiv;
var $elm$core$Basics$lt = _Utils_lt;
var $elm$core$Array$initializeHelp = F5(
	function (fn, fromIndex, len, nodeList, tail) {
		initializeHelp:
		while (true) {
			if (fromIndex < 0) {
				return A2(
					$elm$core$Array$builderToArray,
					false,
					{nodeList: nodeList, nodeListSize: (len / $elm$core$Array$branchFactor) | 0, tail: tail});
			} else {
				var leaf = $elm$core$Array$Leaf(
					A3($elm$core$Elm$JsArray$initialize, $elm$core$Array$branchFactor, fromIndex, fn));
				var $temp$fn = fn,
					$temp$fromIndex = fromIndex - $elm$core$Array$branchFactor,
					$temp$len = len,
					$temp$nodeList = A2($elm$core$List$cons, leaf, nodeList),
					$temp$tail = tail;
				fn = $temp$fn;
				fromIndex = $temp$fromIndex;
				len = $temp$len;
				nodeList = $temp$nodeList;
				tail = $temp$tail;
				continue initializeHelp;
			}
		}
	});
var $elm$core$Basics$remainderBy = _Basics_remainderBy;
var $elm$core$Array$initialize = F2(
	function (len, fn) {
		if (len <= 0) {
			return $elm$core$Array$empty;
		} else {
			var tailLen = len % $elm$core$Array$branchFactor;
			var tail = A3($elm$core$Elm$JsArray$initialize, tailLen, len - tailLen, fn);
			var initialFromIndex = (len - tailLen) - $elm$core$Array$branchFactor;
			return A5($elm$core$Array$initializeHelp, fn, initialFromIndex, len, _List_Nil, tail);
		}
	});
var $elm$core$Basics$True = {$: 'True'};
var $elm$core$Result$isOk = function (result) {
	if (result.$ === 'Ok') {
		return true;
	} else {
		return false;
	}
};
var $elm$json$Json$Decode$map = _Json_map1;
var $elm$json$Json$Decode$map2 = _Json_map2;
var $elm$json$Json$Decode$succeed = _Json_succeed;
var $elm$virtual_dom$VirtualDom$toHandlerInt = function (handler) {
	switch (handler.$) {
		case 'Normal':
			return 0;
		case 'MayStopPropagation':
			return 1;
		case 'MayPreventDefault':
			return 2;
		default:
			return 3;
	}
};
var $elm$browser$Browser$External = function (a) {
	return {$: 'External', a: a};
};
var $elm$browser$Browser$Internal = function (a) {
	return {$: 'Internal', a: a};
};
var $elm$core$Basics$identity = function (x) {
	return x;
};
var $elm$browser$Browser$Dom$NotFound = function (a) {
	return {$: 'NotFound', a: a};
};
var $elm$url$Url$Http = {$: 'Http'};
var $elm$url$Url$Https = {$: 'Https'};
var $elm$url$Url$Url = F6(
	function (protocol, host, port_, path, query, fragment) {
		return {fragment: fragment, host: host, path: path, port_: port_, protocol: protocol, query: query};
	});
var $elm$core$String$contains = _String_contains;
var $elm$core$String$length = _String_length;
var $elm$core$String$slice = _String_slice;
var $elm$core$String$dropLeft = F2(
	function (n, string) {
		return (n < 1) ? string : A3(
			$elm$core$String$slice,
			n,
			$elm$core$String$length(string),
			string);
	});
var $elm$core$String$indexes = _String_indexes;
var $elm$core$String$isEmpty = function (string) {
	return string === '';
};
var $elm$core$String$left = F2(
	function (n, string) {
		return (n < 1) ? '' : A3($elm$core$String$slice, 0, n, string);
	});
var $elm$core$String$toInt = _String_toInt;
var $elm$url$Url$chompBeforePath = F5(
	function (protocol, path, params, frag, str) {
		if ($elm$core$String$isEmpty(str) || A2($elm$core$String$contains, '@', str)) {
			return $elm$core$Maybe$Nothing;
		} else {
			var _v0 = A2($elm$core$String$indexes, ':', str);
			if (!_v0.b) {
				return $elm$core$Maybe$Just(
					A6($elm$url$Url$Url, protocol, str, $elm$core$Maybe$Nothing, path, params, frag));
			} else {
				if (!_v0.b.b) {
					var i = _v0.a;
					var _v1 = $elm$core$String$toInt(
						A2($elm$core$String$dropLeft, i + 1, str));
					if (_v1.$ === 'Nothing') {
						return $elm$core$Maybe$Nothing;
					} else {
						var port_ = _v1;
						return $elm$core$Maybe$Just(
							A6(
								$elm$url$Url$Url,
								protocol,
								A2($elm$core$String$left, i, str),
								port_,
								path,
								params,
								frag));
					}
				} else {
					return $elm$core$Maybe$Nothing;
				}
			}
		}
	});
var $elm$url$Url$chompBeforeQuery = F4(
	function (protocol, params, frag, str) {
		if ($elm$core$String$isEmpty(str)) {
			return $elm$core$Maybe$Nothing;
		} else {
			var _v0 = A2($elm$core$String$indexes, '/', str);
			if (!_v0.b) {
				return A5($elm$url$Url$chompBeforePath, protocol, '/', params, frag, str);
			} else {
				var i = _v0.a;
				return A5(
					$elm$url$Url$chompBeforePath,
					protocol,
					A2($elm$core$String$dropLeft, i, str),
					params,
					frag,
					A2($elm$core$String$left, i, str));
			}
		}
	});
var $elm$url$Url$chompBeforeFragment = F3(
	function (protocol, frag, str) {
		if ($elm$core$String$isEmpty(str)) {
			return $elm$core$Maybe$Nothing;
		} else {
			var _v0 = A2($elm$core$String$indexes, '?', str);
			if (!_v0.b) {
				return A4($elm$url$Url$chompBeforeQuery, protocol, $elm$core$Maybe$Nothing, frag, str);
			} else {
				var i = _v0.a;
				return A4(
					$elm$url$Url$chompBeforeQuery,
					protocol,
					$elm$core$Maybe$Just(
						A2($elm$core$String$dropLeft, i + 1, str)),
					frag,
					A2($elm$core$String$left, i, str));
			}
		}
	});
var $elm$url$Url$chompAfterProtocol = F2(
	function (protocol, str) {
		if ($elm$core$String$isEmpty(str)) {
			return $elm$core$Maybe$Nothing;
		} else {
			var _v0 = A2($elm$core$String$indexes, '#', str);
			if (!_v0.b) {
				return A3($elm$url$Url$chompBeforeFragment, protocol, $elm$core$Maybe$Nothing, str);
			} else {
				var i = _v0.a;
				return A3(
					$elm$url$Url$chompBeforeFragment,
					protocol,
					$elm$core$Maybe$Just(
						A2($elm$core$String$dropLeft, i + 1, str)),
					A2($elm$core$String$left, i, str));
			}
		}
	});
var $elm$core$String$startsWith = _String_startsWith;
var $elm$url$Url$fromString = function (str) {
	return A2($elm$core$String$startsWith, 'http://', str) ? A2(
		$elm$url$Url$chompAfterProtocol,
		$elm$url$Url$Http,
		A2($elm$core$String$dropLeft, 7, str)) : (A2($elm$core$String$startsWith, 'https://', str) ? A2(
		$elm$url$Url$chompAfterProtocol,
		$elm$url$Url$Https,
		A2($elm$core$String$dropLeft, 8, str)) : $elm$core$Maybe$Nothing);
};
var $elm$core$Basics$never = function (_v0) {
	never:
	while (true) {
		var nvr = _v0.a;
		var $temp$_v0 = nvr;
		_v0 = $temp$_v0;
		continue never;
	}
};
var $elm$core$Task$Perform = function (a) {
	return {$: 'Perform', a: a};
};
var $elm$core$Task$succeed = _Scheduler_succeed;
var $elm$core$Task$init = $elm$core$Task$succeed(_Utils_Tuple0);
var $elm$core$List$foldrHelper = F4(
	function (fn, acc, ctr, ls) {
		if (!ls.b) {
			return acc;
		} else {
			var a = ls.a;
			var r1 = ls.b;
			if (!r1.b) {
				return A2(fn, a, acc);
			} else {
				var b = r1.a;
				var r2 = r1.b;
				if (!r2.b) {
					return A2(
						fn,
						a,
						A2(fn, b, acc));
				} else {
					var c = r2.a;
					var r3 = r2.b;
					if (!r3.b) {
						return A2(
							fn,
							a,
							A2(
								fn,
								b,
								A2(fn, c, acc)));
					} else {
						var d = r3.a;
						var r4 = r3.b;
						var res = (ctr > 500) ? A3(
							$elm$core$List$foldl,
							fn,
							acc,
							$elm$core$List$reverse(r4)) : A4($elm$core$List$foldrHelper, fn, acc, ctr + 1, r4);
						return A2(
							fn,
							a,
							A2(
								fn,
								b,
								A2(
									fn,
									c,
									A2(fn, d, res))));
					}
				}
			}
		}
	});
var $elm$core$List$foldr = F3(
	function (fn, acc, ls) {
		return A4($elm$core$List$foldrHelper, fn, acc, 0, ls);
	});
var $elm$core$List$map = F2(
	function (f, xs) {
		return A3(
			$elm$core$List$foldr,
			F2(
				function (x, acc) {
					return A2(
						$elm$core$List$cons,
						f(x),
						acc);
				}),
			_List_Nil,
			xs);
	});
var $elm$core$Task$andThen = _Scheduler_andThen;
var $elm$core$Task$map = F2(
	function (func, taskA) {
		return A2(
			$elm$core$Task$andThen,
			function (a) {
				return $elm$core$Task$succeed(
					func(a));
			},
			taskA);
	});
var $elm$core$Task$map2 = F3(
	function (func, taskA, taskB) {
		return A2(
			$elm$core$Task$andThen,
			function (a) {
				return A2(
					$elm$core$Task$andThen,
					function (b) {
						return $elm$core$Task$succeed(
							A2(func, a, b));
					},
					taskB);
			},
			taskA);
	});
var $elm$core$Task$sequence = function (tasks) {
	return A3(
		$elm$core$List$foldr,
		$elm$core$Task$map2($elm$core$List$cons),
		$elm$core$Task$succeed(_List_Nil),
		tasks);
};
var $elm$core$Platform$sendToApp = _Platform_sendToApp;
var $elm$core$Task$spawnCmd = F2(
	function (router, _v0) {
		var task = _v0.a;
		return _Scheduler_spawn(
			A2(
				$elm$core$Task$andThen,
				$elm$core$Platform$sendToApp(router),
				task));
	});
var $elm$core$Task$onEffects = F3(
	function (router, commands, state) {
		return A2(
			$elm$core$Task$map,
			function (_v0) {
				return _Utils_Tuple0;
			},
			$elm$core$Task$sequence(
				A2(
					$elm$core$List$map,
					$elm$core$Task$spawnCmd(router),
					commands)));
	});
var $elm$core$Task$onSelfMsg = F3(
	function (_v0, _v1, _v2) {
		return $elm$core$Task$succeed(_Utils_Tuple0);
	});
var $elm$core$Task$cmdMap = F2(
	function (tagger, _v0) {
		var task = _v0.a;
		return $elm$core$Task$Perform(
			A2($elm$core$Task$map, tagger, task));
	});
_Platform_effectManagers['Task'] = _Platform_createManager($elm$core$Task$init, $elm$core$Task$onEffects, $elm$core$Task$onSelfMsg, $elm$core$Task$cmdMap);
var $elm$core$Task$command = _Platform_leaf('Task');
var $elm$core$Task$perform = F2(
	function (toMessage, task) {
		return $elm$core$Task$command(
			$elm$core$Task$Perform(
				A2($elm$core$Task$map, toMessage, task)));
	});
var $elm$browser$Browser$element = _Browser_element;
var $author$project$Model$ChangeTab = function (a) {
	return {$: 'ChangeTab', a: a};
};
var $author$project$Model$Events = {$: 'Events'};
var $author$project$Model$Ingredients = function (a) {
	return {$: 'Ingredients', a: a};
};
var $author$project$Model$Model = F4(
	function (tabs, ingredients, recipes, events) {
		return {events: events, ingredients: ingredients, recipes: recipes, tabs: tabs};
	});
var $author$project$Model$Recipes = function (a) {
	return {$: 'Recipes', a: a};
};
var $elm$core$Basics$always = F2(
	function (a, _v0) {
		return a;
	});
var $author$project$Utils$Cursor$create = F2(
	function (a, r) {
		return {active: a, left: _List_Nil, right: r};
	});
var $author$project$Events$Data = function (a) {
	return {$: 'Data', a: a};
};
var $author$project$Utils$Model$NotAsked = {$: 'NotAsked'};
var $author$project$Events$emptyEventsData = $author$project$Events$Data(
	{eventModal: $elm$core$Maybe$Nothing, events: $author$project$Utils$Model$NotAsked});
var $author$project$Ingredients$Model$NoModal = {$: 'NoModal'};
var $author$project$Ingredients$Model$emptyIngredientsTabData = {filter: '', ingredients: $author$project$Utils$Model$NotAsked, modal: $author$project$Ingredients$Model$NoModal};
var $author$project$Recipes$Model$NoModal = {$: 'NoModal'};
var $author$project$Recipes$Model$emptyRecipeTabData = {allIngredients: $author$project$Utils$Model$NotAsked, allUnits: $author$project$Utils$Model$NotAsked, filter: '', modal: $author$project$Recipes$Model$NoModal, recipes: $author$project$Utils$Model$NotAsked};
var $elm$core$Platform$Cmd$map = _Platform_map;
var $elm$core$Platform$Cmd$batch = _Platform_batch;
var $elm$core$Platform$Cmd$none = $elm$core$Platform$Cmd$batch(_List_Nil);
var $author$project$Main$init = function (_v0) {
	var _v1 = _Utils_Tuple3($author$project$Ingredients$Model$emptyIngredientsTabData, $author$project$Recipes$Model$emptyRecipeTabData, $author$project$Events$emptyEventsData);
	var ingredientsTabData = _v1.a;
	var recipeTabData = _v1.b;
	var eventsData = _v1.c;
	var tabs = A2(
		$author$project$Utils$Cursor$create,
		$author$project$Model$Ingredients(ingredientsTabData),
		_List_fromArray(
			[
				$author$project$Model$Recipes(recipeTabData),
				$author$project$Model$Events
			]));
	return _Utils_Tuple2(
		A4($author$project$Model$Model, tabs, ingredientsTabData, recipeTabData, eventsData),
		A2(
			$elm$core$Platform$Cmd$map,
			$elm$core$Basics$always(
				$author$project$Model$ChangeTab(
					$author$project$Model$Ingredients($author$project$Ingredients$Model$emptyIngredientsTabData))),
			$elm$core$Platform$Cmd$none));
};
var $elm$core$Platform$Sub$batch = _Platform_batch;
var $elm$core$Platform$Sub$none = $elm$core$Platform$Sub$batch(_List_Nil);
var $author$project$Main$subscriptions = function (_v0) {
	return $elm$core$Platform$Sub$none;
};
var $author$project$Model$EventsMessage = function (a) {
	return {$: 'EventsMessage', a: a};
};
var $author$project$Model$IngredientMessage = function (a) {
	return {$: 'IngredientMessage', a: a};
};
var $author$project$Ingredients$Model$InitTab = {$: 'InitTab'};
var $author$project$Recipes$Model$InitTab = {$: 'InitTab'};
var $author$project$Model$RecipeMessage = function (a) {
	return {$: 'RecipeMessage', a: a};
};
var $author$project$Events$Details = function (a) {
	return {$: 'Details', a: a};
};
var $author$project$Events$EventListMsg = function (a) {
	return {$: 'EventListMsg', a: a};
};
var $author$project$Utils$Model$Success = function (a) {
	return {$: 'Success', a: a};
};
var $author$project$Events$EventList = function (a) {
	return {$: 'EventList', a: a};
};
var $author$project$Events$GotWebData = function (a) {
	return {$: 'GotWebData', a: a};
};
var $author$project$Settings$backend = function (path) {
	return 'http://localhost:3000' + path;
};
var $elm$core$Basics$composeL = F3(
	function (g, f, x) {
		return g(
			f(x));
	});
var $elm$http$Http$BadStatus_ = F2(
	function (a, b) {
		return {$: 'BadStatus_', a: a, b: b};
	});
var $elm$http$Http$BadUrl_ = function (a) {
	return {$: 'BadUrl_', a: a};
};
var $elm$http$Http$GoodStatus_ = F2(
	function (a, b) {
		return {$: 'GoodStatus_', a: a, b: b};
	});
var $elm$http$Http$NetworkError_ = {$: 'NetworkError_'};
var $elm$http$Http$Receiving = function (a) {
	return {$: 'Receiving', a: a};
};
var $elm$http$Http$Sending = function (a) {
	return {$: 'Sending', a: a};
};
var $elm$http$Http$Timeout_ = {$: 'Timeout_'};
var $elm$core$Dict$RBEmpty_elm_builtin = {$: 'RBEmpty_elm_builtin'};
var $elm$core$Dict$empty = $elm$core$Dict$RBEmpty_elm_builtin;
var $elm$core$Maybe$isJust = function (maybe) {
	if (maybe.$ === 'Just') {
		return true;
	} else {
		return false;
	}
};
var $elm$core$Platform$sendToSelf = _Platform_sendToSelf;
var $elm$core$Basics$compare = _Utils_compare;
var $elm$core$Dict$get = F2(
	function (targetKey, dict) {
		get:
		while (true) {
			if (dict.$ === 'RBEmpty_elm_builtin') {
				return $elm$core$Maybe$Nothing;
			} else {
				var key = dict.b;
				var value = dict.c;
				var left = dict.d;
				var right = dict.e;
				var _v1 = A2($elm$core$Basics$compare, targetKey, key);
				switch (_v1.$) {
					case 'LT':
						var $temp$targetKey = targetKey,
							$temp$dict = left;
						targetKey = $temp$targetKey;
						dict = $temp$dict;
						continue get;
					case 'EQ':
						return $elm$core$Maybe$Just(value);
					default:
						var $temp$targetKey = targetKey,
							$temp$dict = right;
						targetKey = $temp$targetKey;
						dict = $temp$dict;
						continue get;
				}
			}
		}
	});
var $elm$core$Dict$Black = {$: 'Black'};
var $elm$core$Dict$RBNode_elm_builtin = F5(
	function (a, b, c, d, e) {
		return {$: 'RBNode_elm_builtin', a: a, b: b, c: c, d: d, e: e};
	});
var $elm$core$Dict$Red = {$: 'Red'};
var $elm$core$Dict$balance = F5(
	function (color, key, value, left, right) {
		if ((right.$ === 'RBNode_elm_builtin') && (right.a.$ === 'Red')) {
			var _v1 = right.a;
			var rK = right.b;
			var rV = right.c;
			var rLeft = right.d;
			var rRight = right.e;
			if ((left.$ === 'RBNode_elm_builtin') && (left.a.$ === 'Red')) {
				var _v3 = left.a;
				var lK = left.b;
				var lV = left.c;
				var lLeft = left.d;
				var lRight = left.e;
				return A5(
					$elm$core$Dict$RBNode_elm_builtin,
					$elm$core$Dict$Red,
					key,
					value,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Black, lK, lV, lLeft, lRight),
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Black, rK, rV, rLeft, rRight));
			} else {
				return A5(
					$elm$core$Dict$RBNode_elm_builtin,
					color,
					rK,
					rV,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, key, value, left, rLeft),
					rRight);
			}
		} else {
			if ((((left.$ === 'RBNode_elm_builtin') && (left.a.$ === 'Red')) && (left.d.$ === 'RBNode_elm_builtin')) && (left.d.a.$ === 'Red')) {
				var _v5 = left.a;
				var lK = left.b;
				var lV = left.c;
				var _v6 = left.d;
				var _v7 = _v6.a;
				var llK = _v6.b;
				var llV = _v6.c;
				var llLeft = _v6.d;
				var llRight = _v6.e;
				var lRight = left.e;
				return A5(
					$elm$core$Dict$RBNode_elm_builtin,
					$elm$core$Dict$Red,
					lK,
					lV,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Black, llK, llV, llLeft, llRight),
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Black, key, value, lRight, right));
			} else {
				return A5($elm$core$Dict$RBNode_elm_builtin, color, key, value, left, right);
			}
		}
	});
var $elm$core$Dict$insertHelp = F3(
	function (key, value, dict) {
		if (dict.$ === 'RBEmpty_elm_builtin') {
			return A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, key, value, $elm$core$Dict$RBEmpty_elm_builtin, $elm$core$Dict$RBEmpty_elm_builtin);
		} else {
			var nColor = dict.a;
			var nKey = dict.b;
			var nValue = dict.c;
			var nLeft = dict.d;
			var nRight = dict.e;
			var _v1 = A2($elm$core$Basics$compare, key, nKey);
			switch (_v1.$) {
				case 'LT':
					return A5(
						$elm$core$Dict$balance,
						nColor,
						nKey,
						nValue,
						A3($elm$core$Dict$insertHelp, key, value, nLeft),
						nRight);
				case 'EQ':
					return A5($elm$core$Dict$RBNode_elm_builtin, nColor, nKey, value, nLeft, nRight);
				default:
					return A5(
						$elm$core$Dict$balance,
						nColor,
						nKey,
						nValue,
						nLeft,
						A3($elm$core$Dict$insertHelp, key, value, nRight));
			}
		}
	});
var $elm$core$Dict$insert = F3(
	function (key, value, dict) {
		var _v0 = A3($elm$core$Dict$insertHelp, key, value, dict);
		if ((_v0.$ === 'RBNode_elm_builtin') && (_v0.a.$ === 'Red')) {
			var _v1 = _v0.a;
			var k = _v0.b;
			var v = _v0.c;
			var l = _v0.d;
			var r = _v0.e;
			return A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Black, k, v, l, r);
		} else {
			var x = _v0;
			return x;
		}
	});
var $elm$core$Dict$getMin = function (dict) {
	getMin:
	while (true) {
		if ((dict.$ === 'RBNode_elm_builtin') && (dict.d.$ === 'RBNode_elm_builtin')) {
			var left = dict.d;
			var $temp$dict = left;
			dict = $temp$dict;
			continue getMin;
		} else {
			return dict;
		}
	}
};
var $elm$core$Dict$moveRedLeft = function (dict) {
	if (((dict.$ === 'RBNode_elm_builtin') && (dict.d.$ === 'RBNode_elm_builtin')) && (dict.e.$ === 'RBNode_elm_builtin')) {
		if ((dict.e.d.$ === 'RBNode_elm_builtin') && (dict.e.d.a.$ === 'Red')) {
			var clr = dict.a;
			var k = dict.b;
			var v = dict.c;
			var _v1 = dict.d;
			var lClr = _v1.a;
			var lK = _v1.b;
			var lV = _v1.c;
			var lLeft = _v1.d;
			var lRight = _v1.e;
			var _v2 = dict.e;
			var rClr = _v2.a;
			var rK = _v2.b;
			var rV = _v2.c;
			var rLeft = _v2.d;
			var _v3 = rLeft.a;
			var rlK = rLeft.b;
			var rlV = rLeft.c;
			var rlL = rLeft.d;
			var rlR = rLeft.e;
			var rRight = _v2.e;
			return A5(
				$elm$core$Dict$RBNode_elm_builtin,
				$elm$core$Dict$Red,
				rlK,
				rlV,
				A5(
					$elm$core$Dict$RBNode_elm_builtin,
					$elm$core$Dict$Black,
					k,
					v,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, lK, lV, lLeft, lRight),
					rlL),
				A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Black, rK, rV, rlR, rRight));
		} else {
			var clr = dict.a;
			var k = dict.b;
			var v = dict.c;
			var _v4 = dict.d;
			var lClr = _v4.a;
			var lK = _v4.b;
			var lV = _v4.c;
			var lLeft = _v4.d;
			var lRight = _v4.e;
			var _v5 = dict.e;
			var rClr = _v5.a;
			var rK = _v5.b;
			var rV = _v5.c;
			var rLeft = _v5.d;
			var rRight = _v5.e;
			if (clr.$ === 'Black') {
				return A5(
					$elm$core$Dict$RBNode_elm_builtin,
					$elm$core$Dict$Black,
					k,
					v,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, lK, lV, lLeft, lRight),
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, rK, rV, rLeft, rRight));
			} else {
				return A5(
					$elm$core$Dict$RBNode_elm_builtin,
					$elm$core$Dict$Black,
					k,
					v,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, lK, lV, lLeft, lRight),
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, rK, rV, rLeft, rRight));
			}
		}
	} else {
		return dict;
	}
};
var $elm$core$Dict$moveRedRight = function (dict) {
	if (((dict.$ === 'RBNode_elm_builtin') && (dict.d.$ === 'RBNode_elm_builtin')) && (dict.e.$ === 'RBNode_elm_builtin')) {
		if ((dict.d.d.$ === 'RBNode_elm_builtin') && (dict.d.d.a.$ === 'Red')) {
			var clr = dict.a;
			var k = dict.b;
			var v = dict.c;
			var _v1 = dict.d;
			var lClr = _v1.a;
			var lK = _v1.b;
			var lV = _v1.c;
			var _v2 = _v1.d;
			var _v3 = _v2.a;
			var llK = _v2.b;
			var llV = _v2.c;
			var llLeft = _v2.d;
			var llRight = _v2.e;
			var lRight = _v1.e;
			var _v4 = dict.e;
			var rClr = _v4.a;
			var rK = _v4.b;
			var rV = _v4.c;
			var rLeft = _v4.d;
			var rRight = _v4.e;
			return A5(
				$elm$core$Dict$RBNode_elm_builtin,
				$elm$core$Dict$Red,
				lK,
				lV,
				A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Black, llK, llV, llLeft, llRight),
				A5(
					$elm$core$Dict$RBNode_elm_builtin,
					$elm$core$Dict$Black,
					k,
					v,
					lRight,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, rK, rV, rLeft, rRight)));
		} else {
			var clr = dict.a;
			var k = dict.b;
			var v = dict.c;
			var _v5 = dict.d;
			var lClr = _v5.a;
			var lK = _v5.b;
			var lV = _v5.c;
			var lLeft = _v5.d;
			var lRight = _v5.e;
			var _v6 = dict.e;
			var rClr = _v6.a;
			var rK = _v6.b;
			var rV = _v6.c;
			var rLeft = _v6.d;
			var rRight = _v6.e;
			if (clr.$ === 'Black') {
				return A5(
					$elm$core$Dict$RBNode_elm_builtin,
					$elm$core$Dict$Black,
					k,
					v,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, lK, lV, lLeft, lRight),
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, rK, rV, rLeft, rRight));
			} else {
				return A5(
					$elm$core$Dict$RBNode_elm_builtin,
					$elm$core$Dict$Black,
					k,
					v,
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, lK, lV, lLeft, lRight),
					A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, rK, rV, rLeft, rRight));
			}
		}
	} else {
		return dict;
	}
};
var $elm$core$Dict$removeHelpPrepEQGT = F7(
	function (targetKey, dict, color, key, value, left, right) {
		if ((left.$ === 'RBNode_elm_builtin') && (left.a.$ === 'Red')) {
			var _v1 = left.a;
			var lK = left.b;
			var lV = left.c;
			var lLeft = left.d;
			var lRight = left.e;
			return A5(
				$elm$core$Dict$RBNode_elm_builtin,
				color,
				lK,
				lV,
				lLeft,
				A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Red, key, value, lRight, right));
		} else {
			_v2$2:
			while (true) {
				if ((right.$ === 'RBNode_elm_builtin') && (right.a.$ === 'Black')) {
					if (right.d.$ === 'RBNode_elm_builtin') {
						if (right.d.a.$ === 'Black') {
							var _v3 = right.a;
							var _v4 = right.d;
							var _v5 = _v4.a;
							return $elm$core$Dict$moveRedRight(dict);
						} else {
							break _v2$2;
						}
					} else {
						var _v6 = right.a;
						var _v7 = right.d;
						return $elm$core$Dict$moveRedRight(dict);
					}
				} else {
					break _v2$2;
				}
			}
			return dict;
		}
	});
var $elm$core$Dict$removeMin = function (dict) {
	if ((dict.$ === 'RBNode_elm_builtin') && (dict.d.$ === 'RBNode_elm_builtin')) {
		var color = dict.a;
		var key = dict.b;
		var value = dict.c;
		var left = dict.d;
		var lColor = left.a;
		var lLeft = left.d;
		var right = dict.e;
		if (lColor.$ === 'Black') {
			if ((lLeft.$ === 'RBNode_elm_builtin') && (lLeft.a.$ === 'Red')) {
				var _v3 = lLeft.a;
				return A5(
					$elm$core$Dict$RBNode_elm_builtin,
					color,
					key,
					value,
					$elm$core$Dict$removeMin(left),
					right);
			} else {
				var _v4 = $elm$core$Dict$moveRedLeft(dict);
				if (_v4.$ === 'RBNode_elm_builtin') {
					var nColor = _v4.a;
					var nKey = _v4.b;
					var nValue = _v4.c;
					var nLeft = _v4.d;
					var nRight = _v4.e;
					return A5(
						$elm$core$Dict$balance,
						nColor,
						nKey,
						nValue,
						$elm$core$Dict$removeMin(nLeft),
						nRight);
				} else {
					return $elm$core$Dict$RBEmpty_elm_builtin;
				}
			}
		} else {
			return A5(
				$elm$core$Dict$RBNode_elm_builtin,
				color,
				key,
				value,
				$elm$core$Dict$removeMin(left),
				right);
		}
	} else {
		return $elm$core$Dict$RBEmpty_elm_builtin;
	}
};
var $elm$core$Dict$removeHelp = F2(
	function (targetKey, dict) {
		if (dict.$ === 'RBEmpty_elm_builtin') {
			return $elm$core$Dict$RBEmpty_elm_builtin;
		} else {
			var color = dict.a;
			var key = dict.b;
			var value = dict.c;
			var left = dict.d;
			var right = dict.e;
			if (_Utils_cmp(targetKey, key) < 0) {
				if ((left.$ === 'RBNode_elm_builtin') && (left.a.$ === 'Black')) {
					var _v4 = left.a;
					var lLeft = left.d;
					if ((lLeft.$ === 'RBNode_elm_builtin') && (lLeft.a.$ === 'Red')) {
						var _v6 = lLeft.a;
						return A5(
							$elm$core$Dict$RBNode_elm_builtin,
							color,
							key,
							value,
							A2($elm$core$Dict$removeHelp, targetKey, left),
							right);
					} else {
						var _v7 = $elm$core$Dict$moveRedLeft(dict);
						if (_v7.$ === 'RBNode_elm_builtin') {
							var nColor = _v7.a;
							var nKey = _v7.b;
							var nValue = _v7.c;
							var nLeft = _v7.d;
							var nRight = _v7.e;
							return A5(
								$elm$core$Dict$balance,
								nColor,
								nKey,
								nValue,
								A2($elm$core$Dict$removeHelp, targetKey, nLeft),
								nRight);
						} else {
							return $elm$core$Dict$RBEmpty_elm_builtin;
						}
					}
				} else {
					return A5(
						$elm$core$Dict$RBNode_elm_builtin,
						color,
						key,
						value,
						A2($elm$core$Dict$removeHelp, targetKey, left),
						right);
				}
			} else {
				return A2(
					$elm$core$Dict$removeHelpEQGT,
					targetKey,
					A7($elm$core$Dict$removeHelpPrepEQGT, targetKey, dict, color, key, value, left, right));
			}
		}
	});
var $elm$core$Dict$removeHelpEQGT = F2(
	function (targetKey, dict) {
		if (dict.$ === 'RBNode_elm_builtin') {
			var color = dict.a;
			var key = dict.b;
			var value = dict.c;
			var left = dict.d;
			var right = dict.e;
			if (_Utils_eq(targetKey, key)) {
				var _v1 = $elm$core$Dict$getMin(right);
				if (_v1.$ === 'RBNode_elm_builtin') {
					var minKey = _v1.b;
					var minValue = _v1.c;
					return A5(
						$elm$core$Dict$balance,
						color,
						minKey,
						minValue,
						left,
						$elm$core$Dict$removeMin(right));
				} else {
					return $elm$core$Dict$RBEmpty_elm_builtin;
				}
			} else {
				return A5(
					$elm$core$Dict$balance,
					color,
					key,
					value,
					left,
					A2($elm$core$Dict$removeHelp, targetKey, right));
			}
		} else {
			return $elm$core$Dict$RBEmpty_elm_builtin;
		}
	});
var $elm$core$Dict$remove = F2(
	function (key, dict) {
		var _v0 = A2($elm$core$Dict$removeHelp, key, dict);
		if ((_v0.$ === 'RBNode_elm_builtin') && (_v0.a.$ === 'Red')) {
			var _v1 = _v0.a;
			var k = _v0.b;
			var v = _v0.c;
			var l = _v0.d;
			var r = _v0.e;
			return A5($elm$core$Dict$RBNode_elm_builtin, $elm$core$Dict$Black, k, v, l, r);
		} else {
			var x = _v0;
			return x;
		}
	});
var $elm$core$Dict$update = F3(
	function (targetKey, alter, dictionary) {
		var _v0 = alter(
			A2($elm$core$Dict$get, targetKey, dictionary));
		if (_v0.$ === 'Just') {
			var value = _v0.a;
			return A3($elm$core$Dict$insert, targetKey, value, dictionary);
		} else {
			return A2($elm$core$Dict$remove, targetKey, dictionary);
		}
	});
var $elm$http$Http$emptyBody = _Http_emptyBody;
var $author$project$Events$Exists = function (a) {
	return {$: 'Exists', a: a};
};
var $elm$json$Json$Decode$field = _Json_decodeField;
var $elm$json$Json$Decode$int = _Json_decodeInt;
var $elm$json$Json$Decode$map4 = _Json_map4;
var $elm$json$Json$Decode$oneOf = _Json_oneOf;
var $elm$json$Json$Decode$maybe = function (decoder) {
	return $elm$json$Json$Decode$oneOf(
		_List_fromArray(
			[
				A2($elm$json$Json$Decode$map, $elm$core$Maybe$Just, decoder),
				$elm$json$Json$Decode$succeed($elm$core$Maybe$Nothing)
			]));
};
var $elm$json$Json$Decode$string = _Json_decodeString;
var $author$project$Events$eventDecoder = function () {
	var _new = F4(
		function (name, budget, id, comment) {
			return $author$project$Events$Exists(
				{budget: budget, comment: comment, id: id, name: name});
		});
	return A5(
		$elm$json$Json$Decode$map4,
		_new,
		A2($elm$json$Json$Decode$field, 'name', $elm$json$Json$Decode$string),
		A2($elm$json$Json$Decode$field, 'budget', $elm$json$Json$Decode$string),
		A2($elm$json$Json$Decode$field, 'id', $elm$json$Json$Decode$int),
		A2(
			$elm$json$Json$Decode$field,
			'comment',
			$elm$json$Json$Decode$maybe($elm$json$Json$Decode$string)));
}();
var $elm$json$Json$Decode$list = _Json_decodeList;
var $author$project$Events$eventListDecoder = $elm$json$Json$Decode$list($author$project$Events$eventDecoder);
var $elm$json$Json$Decode$decodeString = _Json_runOnString;
var $elm$core$Basics$composeR = F3(
	function (f, g, x) {
		return g(
			f(x));
	});
var $elm$http$Http$expectStringResponse = F2(
	function (toMsg, toResult) {
		return A3(
			_Http_expect,
			'',
			$elm$core$Basics$identity,
			A2($elm$core$Basics$composeR, toResult, toMsg));
	});
var $elm$core$Result$mapError = F2(
	function (f, result) {
		if (result.$ === 'Ok') {
			var v = result.a;
			return $elm$core$Result$Ok(v);
		} else {
			var e = result.a;
			return $elm$core$Result$Err(
				f(e));
		}
	});
var $elm$http$Http$BadBody = function (a) {
	return {$: 'BadBody', a: a};
};
var $elm$http$Http$BadStatus = function (a) {
	return {$: 'BadStatus', a: a};
};
var $elm$http$Http$BadUrl = function (a) {
	return {$: 'BadUrl', a: a};
};
var $elm$http$Http$NetworkError = {$: 'NetworkError'};
var $elm$http$Http$Timeout = {$: 'Timeout'};
var $elm$http$Http$resolve = F2(
	function (toResult, response) {
		switch (response.$) {
			case 'BadUrl_':
				var url = response.a;
				return $elm$core$Result$Err(
					$elm$http$Http$BadUrl(url));
			case 'Timeout_':
				return $elm$core$Result$Err($elm$http$Http$Timeout);
			case 'NetworkError_':
				return $elm$core$Result$Err($elm$http$Http$NetworkError);
			case 'BadStatus_':
				var metadata = response.a;
				return $elm$core$Result$Err(
					$elm$http$Http$BadStatus(metadata.statusCode));
			default:
				var body = response.b;
				return A2(
					$elm$core$Result$mapError,
					$elm$http$Http$BadBody,
					toResult(body));
		}
	});
var $elm$http$Http$expectJson = F2(
	function (toMsg, decoder) {
		return A2(
			$elm$http$Http$expectStringResponse,
			toMsg,
			$elm$http$Http$resolve(
				function (string) {
					return A2(
						$elm$core$Result$mapError,
						$elm$json$Json$Decode$errorToString,
						A2($elm$json$Json$Decode$decodeString, decoder, string));
				}));
	});
var $elm$http$Http$Request = function (a) {
	return {$: 'Request', a: a};
};
var $elm$http$Http$State = F2(
	function (reqs, subs) {
		return {reqs: reqs, subs: subs};
	});
var $elm$http$Http$init = $elm$core$Task$succeed(
	A2($elm$http$Http$State, $elm$core$Dict$empty, _List_Nil));
var $elm$core$Process$kill = _Scheduler_kill;
var $elm$core$Process$spawn = _Scheduler_spawn;
var $elm$http$Http$updateReqs = F3(
	function (router, cmds, reqs) {
		updateReqs:
		while (true) {
			if (!cmds.b) {
				return $elm$core$Task$succeed(reqs);
			} else {
				var cmd = cmds.a;
				var otherCmds = cmds.b;
				if (cmd.$ === 'Cancel') {
					var tracker = cmd.a;
					var _v2 = A2($elm$core$Dict$get, tracker, reqs);
					if (_v2.$ === 'Nothing') {
						var $temp$router = router,
							$temp$cmds = otherCmds,
							$temp$reqs = reqs;
						router = $temp$router;
						cmds = $temp$cmds;
						reqs = $temp$reqs;
						continue updateReqs;
					} else {
						var pid = _v2.a;
						return A2(
							$elm$core$Task$andThen,
							function (_v3) {
								return A3(
									$elm$http$Http$updateReqs,
									router,
									otherCmds,
									A2($elm$core$Dict$remove, tracker, reqs));
							},
							$elm$core$Process$kill(pid));
					}
				} else {
					var req = cmd.a;
					return A2(
						$elm$core$Task$andThen,
						function (pid) {
							var _v4 = req.tracker;
							if (_v4.$ === 'Nothing') {
								return A3($elm$http$Http$updateReqs, router, otherCmds, reqs);
							} else {
								var tracker = _v4.a;
								return A3(
									$elm$http$Http$updateReqs,
									router,
									otherCmds,
									A3($elm$core$Dict$insert, tracker, pid, reqs));
							}
						},
						$elm$core$Process$spawn(
							A3(
								_Http_toTask,
								router,
								$elm$core$Platform$sendToApp(router),
								req)));
				}
			}
		}
	});
var $elm$http$Http$onEffects = F4(
	function (router, cmds, subs, state) {
		return A2(
			$elm$core$Task$andThen,
			function (reqs) {
				return $elm$core$Task$succeed(
					A2($elm$http$Http$State, reqs, subs));
			},
			A3($elm$http$Http$updateReqs, router, cmds, state.reqs));
	});
var $elm$core$List$maybeCons = F3(
	function (f, mx, xs) {
		var _v0 = f(mx);
		if (_v0.$ === 'Just') {
			var x = _v0.a;
			return A2($elm$core$List$cons, x, xs);
		} else {
			return xs;
		}
	});
var $elm$core$List$filterMap = F2(
	function (f, xs) {
		return A3(
			$elm$core$List$foldr,
			$elm$core$List$maybeCons(f),
			_List_Nil,
			xs);
	});
var $elm$http$Http$maybeSend = F4(
	function (router, desiredTracker, progress, _v0) {
		var actualTracker = _v0.a;
		var toMsg = _v0.b;
		return _Utils_eq(desiredTracker, actualTracker) ? $elm$core$Maybe$Just(
			A2(
				$elm$core$Platform$sendToApp,
				router,
				toMsg(progress))) : $elm$core$Maybe$Nothing;
	});
var $elm$http$Http$onSelfMsg = F3(
	function (router, _v0, state) {
		var tracker = _v0.a;
		var progress = _v0.b;
		return A2(
			$elm$core$Task$andThen,
			function (_v1) {
				return $elm$core$Task$succeed(state);
			},
			$elm$core$Task$sequence(
				A2(
					$elm$core$List$filterMap,
					A3($elm$http$Http$maybeSend, router, tracker, progress),
					state.subs)));
	});
var $elm$http$Http$Cancel = function (a) {
	return {$: 'Cancel', a: a};
};
var $elm$http$Http$cmdMap = F2(
	function (func, cmd) {
		if (cmd.$ === 'Cancel') {
			var tracker = cmd.a;
			return $elm$http$Http$Cancel(tracker);
		} else {
			var r = cmd.a;
			return $elm$http$Http$Request(
				{
					allowCookiesFromOtherDomains: r.allowCookiesFromOtherDomains,
					body: r.body,
					expect: A2(_Http_mapExpect, func, r.expect),
					headers: r.headers,
					method: r.method,
					timeout: r.timeout,
					tracker: r.tracker,
					url: r.url
				});
		}
	});
var $elm$http$Http$MySub = F2(
	function (a, b) {
		return {$: 'MySub', a: a, b: b};
	});
var $elm$http$Http$subMap = F2(
	function (func, _v0) {
		var tracker = _v0.a;
		var toMsg = _v0.b;
		return A2(
			$elm$http$Http$MySub,
			tracker,
			A2($elm$core$Basics$composeR, toMsg, func));
	});
_Platform_effectManagers['Http'] = _Platform_createManager($elm$http$Http$init, $elm$http$Http$onEffects, $elm$http$Http$onSelfMsg, $elm$http$Http$cmdMap, $elm$http$Http$subMap);
var $elm$http$Http$command = _Platform_leaf('Http');
var $elm$http$Http$subscription = _Platform_leaf('Http');
var $elm$http$Http$request = function (r) {
	return $elm$http$Http$command(
		$elm$http$Http$Request(
			{allowCookiesFromOtherDomains: false, body: r.body, expect: r.expect, headers: r.headers, method: r.method, timeout: r.timeout, tracker: r.tracker, url: r.url}));
};
var $author$project$Events$deleteEvent = function (id) {
	return $elm$http$Http$request(
		{
			body: $elm$http$Http$emptyBody,
			expect: A2(
				$elm$http$Http$expectJson,
				A2($elm$core$Basics$composeL, $author$project$Events$GotWebData, $author$project$Events$EventList),
				$author$project$Events$eventListDecoder),
			headers: _List_Nil,
			method: 'DELETE',
			timeout: $elm$core$Maybe$Nothing,
			tracker: $elm$core$Maybe$Nothing,
			url: $author$project$Settings$backend(
				'/events/' + $elm$core$String$fromInt(id))
		});
};
var $elm$http$Http$get = function (r) {
	return $elm$http$Http$request(
		{body: $elm$http$Http$emptyBody, expect: r.expect, headers: _List_Nil, method: 'GET', timeout: $elm$core$Maybe$Nothing, tracker: $elm$core$Maybe$Nothing, url: r.url});
};
var $author$project$Events$fetchEvents = $elm$http$Http$get(
	{
		expect: A2(
			$elm$http$Http$expectJson,
			A2($elm$core$Basics$composeL, $author$project$Events$GotWebData, $author$project$Events$EventList),
			$author$project$Events$eventListDecoder),
		url: $author$project$Settings$backend('/events/list')
	});
var $author$project$Events$fetchMeals = function (id) {
	return $elm$http$Http$get(
		{
			expect: A2(
				$elm$http$Http$expectJson,
				A2($elm$core$Basics$composeL, $author$project$Events$GotWebData, $author$project$Events$EventList),
				$author$project$Events$eventListDecoder),
			url: $author$project$Settings$backend(
				'/events/' + ($elm$core$String$fromInt(id) + '/meals/list'))
		});
};
var $author$project$Events$getEvent = function (details) {
	var event = details.a.event;
	return event;
};
var $author$project$Events$NewEvent = function (a) {
	return {$: 'NewEvent', a: a};
};
var $author$project$Events$setEventBudget = F2(
	function (event, budget) {
		if (event.$ === 'Exists') {
			var name = event.a.name;
			var id = event.a.id;
			var comment = event.a.comment;
			return $author$project$Events$Exists(
				{budget: budget, comment: comment, id: id, name: name});
		} else {
			var name = event.a.name;
			var comment = event.a.comment;
			return $author$project$Events$NewEvent(
				{budget: budget, comment: comment, name: name});
		}
	});
var $author$project$Events$setEventComment = F2(
	function (event, comment) {
		if (event.$ === 'Exists') {
			var name = event.a.name;
			var budget = event.a.budget;
			var id = event.a.id;
			return $author$project$Events$Exists(
				{
					budget: budget,
					comment: $elm$core$Maybe$Just(comment),
					id: id,
					name: name
				});
		} else {
			var name = event.a.name;
			var budget = event.a.budget;
			return $author$project$Events$NewEvent(
				{
					budget: budget,
					comment: $elm$core$Maybe$Just(comment),
					name: name
				});
		}
	});
var $author$project$Events$setEventName = F2(
	function (event, name) {
		if (event.$ === 'Exists') {
			var budget = event.a.budget;
			var id = event.a.id;
			var comment = event.a.comment;
			return $author$project$Events$Exists(
				{budget: budget, comment: comment, id: id, name: name});
		} else {
			var budget = event.a.budget;
			var comment = event.a.comment;
			return $author$project$Events$NewEvent(
				{budget: budget, comment: comment, name: name});
		}
	});
var $elm$core$Debug$todo = _Debug_todo;
var $author$project$Events$handleEventDetailsMsg = F2(
	function (ev, msg) {
		var event = ev.a.event;
		var details = ev.a.details;
		var mealModal = ev.a.mealModal;
		switch (msg.$) {
			case 'Name':
				var name = msg.a;
				return _Utils_Tuple2(
					$author$project$Events$Details(
						{
							details: details,
							event: A2($author$project$Events$setEventName, event, name),
							mealModal: mealModal
						}),
					$elm$core$Platform$Cmd$none);
			case 'Budget':
				var budget = msg.a;
				return _Utils_Tuple2(
					$author$project$Events$Details(
						{
							details: details,
							event: A2($author$project$Events$setEventBudget, event, budget),
							mealModal: mealModal
						}),
					$elm$core$Platform$Cmd$none);
			case 'Comment':
				var comment = msg.a;
				return _Utils_Tuple2(
					$author$project$Events$Details(
						{
							details: details,
							event: A2($author$project$Events$setEventComment, event, comment),
							mealModal: mealModal
						}),
					$elm$core$Platform$Cmd$none);
			case 'EditMeal':
				var meal = msg.a;
				return _Utils_Tuple2(
					$author$project$Events$Details(
						{
							details: details,
							event: event,
							mealModal: $elm$core$Maybe$Just(meal)
						}),
					$elm$core$Platform$Cmd$none);
			case 'DeleteMeal':
				var meal = msg.a;
				return _Debug_todo(
					'Events',
					{
						start: {line: 512, column: 21},
						end: {line: 512, column: 31}
					})('MealModification');
			default:
				return _Debug_todo(
					'Events',
					{
						start: {line: 515, column: 21},
						end: {line: 515, column: 31}
					})('MealModification');
		}
	});
var $author$project$SearchList$SearchList = function (a) {
	return {$: 'SearchList', a: a};
};
var $author$project$SearchList$handleMsg = F2(
	function (searchList, msg) {
		var s = searchList.a;
		var newSearch = msg.a;
		return _Utils_Tuple3(
			$elm$core$Platform$Cmd$none,
			$author$project$SearchList$SearchList(
				_Utils_update(
					s,
					{search: newSearch})),
			$elm$core$Platform$Cmd$none);
	});
var $author$project$Utils$Model$Failure = function (a) {
	return {$: 'Failure', a: a};
};
var $author$project$Events$eventName = function (event) {
	if (event.$ === 'Exists') {
		var name = event.a.name;
		return name;
	} else {
		var name = event.a.name;
		return name;
	}
};
var $author$project$Utils$Model$Loading = {$: 'Loading'};
var $author$project$Utils$Main$mapWebdata = F2(
	function (f, wd) {
		switch (wd.$) {
			case 'Success':
				var a = wd.a;
				return $author$project$Utils$Model$Success(
					f(a));
			case 'Failure':
				var e = wd.a;
				return $author$project$Utils$Model$Failure(e);
			case 'NotAsked':
				return $author$project$Utils$Model$NotAsked;
			default:
				return $author$project$Utils$Model$Loading;
		}
	});
var $author$project$SearchList$addAll = F2(
	function (list, searchList) {
		var s = searchList.a;
		return $author$project$SearchList$SearchList(
			_Utils_update(
				s,
				{
					list: _Utils_ap(list, s.list)
				}));
	});
var $author$project$SearchList$empty = F3(
	function (mapMsg, viewFilter, viewContent) {
		return $author$project$SearchList$SearchList(
			{list: _List_Nil, mapMsg: mapMsg, search: '', viewContent: viewContent, viewFilter: viewFilter});
	});
var $author$project$SearchList$new = F4(
	function (mapMsg, viewFilter, viewContent, list) {
		return A2(
			$author$project$SearchList$addAll,
			list,
			A3($author$project$SearchList$empty, mapMsg, viewFilter, viewContent));
	});
var $elm$core$String$toLower = _String_toLower;
var $author$project$Utils$Main$propertyFilter = F3(
	function (property, filter, item) {
		return A2(
			$elm$core$String$contains,
			$elm$core$String$toLower(filter),
			$elm$core$String$toLower(
				property(item)));
	});
var $author$project$Events$DeleteEvent = function (a) {
	return {$: 'DeleteEvent', a: a};
};
var $author$project$Events$OpenModal = function (a) {
	return {$: 'OpenModal', a: a};
};
var $elm$html$Html$a = _VirtualDom_node('a');
var $elm$svg$Svg$Attributes$d = _VirtualDom_attribute('d');
var $feathericons$elm_feather$FeatherIcons$Icon = function (a) {
	return {$: 'Icon', a: a};
};
var $feathericons$elm_feather$FeatherIcons$defaultAttributes = function (name) {
	return {
		_class: $elm$core$Maybe$Just('feather feather-' + name),
		size: 24,
		sizeUnit: '',
		strokeWidth: 2,
		viewBox: '0 0 24 24'
	};
};
var $feathericons$elm_feather$FeatherIcons$makeBuilder = F2(
	function (name, src) {
		return $feathericons$elm_feather$FeatherIcons$Icon(
			{
				attrs: $feathericons$elm_feather$FeatherIcons$defaultAttributes(name),
				src: src
			});
	});
var $elm$svg$Svg$trustedNode = _VirtualDom_nodeNS('http://www.w3.org/2000/svg');
var $elm$svg$Svg$path = $elm$svg$Svg$trustedNode('path');
var $feathericons$elm_feather$FeatherIcons$edit = A2(
	$feathericons$elm_feather$FeatherIcons$makeBuilder,
	'edit',
	_List_fromArray(
		[
			A2(
			$elm$svg$Svg$path,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$d('M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7')
				]),
			_List_Nil),
			A2(
			$elm$svg$Svg$path,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$d('M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z')
				]),
			_List_Nil)
		]));
var $elm$json$Json$Encode$string = _Json_wrap;
var $elm$html$Html$Attributes$stringProperty = F2(
	function (key, string) {
		return A2(
			_VirtualDom_property,
			key,
			$elm$json$Json$Encode$string(string));
	});
var $elm$html$Html$Attributes$href = function (url) {
	return A2(
		$elm$html$Html$Attributes$stringProperty,
		'href',
		_VirtualDom_noJavaScriptUri(url));
};
var $author$project$Events$modalFromEvent = function (event) {
	return $author$project$Events$Details(
		{details: $author$project$Utils$Model$NotAsked, event: event, mealModal: $elm$core$Maybe$Nothing});
};
var $elm$virtual_dom$VirtualDom$Normal = function (a) {
	return {$: 'Normal', a: a};
};
var $elm$virtual_dom$VirtualDom$on = _VirtualDom_on;
var $elm$html$Html$Events$on = F2(
	function (event, decoder) {
		return A2(
			$elm$virtual_dom$VirtualDom$on,
			event,
			$elm$virtual_dom$VirtualDom$Normal(decoder));
	});
var $elm$html$Html$Events$onClick = function (msg) {
	return A2(
		$elm$html$Html$Events$on,
		'click',
		$elm$json$Json$Decode$succeed(msg));
};
var $elm$svg$Svg$line = $elm$svg$Svg$trustedNode('line');
var $elm$svg$Svg$Attributes$x1 = _VirtualDom_attribute('x1');
var $elm$svg$Svg$Attributes$x2 = _VirtualDom_attribute('x2');
var $elm$svg$Svg$Attributes$y1 = _VirtualDom_attribute('y1');
var $elm$svg$Svg$Attributes$y2 = _VirtualDom_attribute('y2');
var $feathericons$elm_feather$FeatherIcons$plus = A2(
	$feathericons$elm_feather$FeatherIcons$makeBuilder,
	'plus',
	_List_fromArray(
		[
			A2(
			$elm$svg$Svg$line,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$x1('12'),
					$elm$svg$Svg$Attributes$y1('5'),
					$elm$svg$Svg$Attributes$x2('12'),
					$elm$svg$Svg$Attributes$y2('19')
				]),
			_List_Nil),
			A2(
			$elm$svg$Svg$line,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$x1('5'),
					$elm$svg$Svg$Attributes$y1('12'),
					$elm$svg$Svg$Attributes$x2('19'),
					$elm$svg$Svg$Attributes$y2('12')
				]),
			_List_Nil)
		]));
var $elm$html$Html$span = _VirtualDom_node('span');
var $elm$virtual_dom$VirtualDom$text = _VirtualDom_text;
var $elm$html$Html$text = $elm$virtual_dom$VirtualDom$text;
var $elm$svg$Svg$Attributes$class = _VirtualDom_attribute('class');
var $elm$svg$Svg$Attributes$fill = _VirtualDom_attribute('fill');
var $elm$core$String$fromFloat = _String_fromNumber;
var $elm$svg$Svg$Attributes$height = _VirtualDom_attribute('height');
var $elm$virtual_dom$VirtualDom$map = _VirtualDom_map;
var $elm$svg$Svg$map = $elm$virtual_dom$VirtualDom$map;
var $elm$svg$Svg$Attributes$stroke = _VirtualDom_attribute('stroke');
var $elm$svg$Svg$Attributes$strokeLinecap = _VirtualDom_attribute('stroke-linecap');
var $elm$svg$Svg$Attributes$strokeLinejoin = _VirtualDom_attribute('stroke-linejoin');
var $elm$svg$Svg$Attributes$strokeWidth = _VirtualDom_attribute('stroke-width');
var $elm$svg$Svg$svg = $elm$svg$Svg$trustedNode('svg');
var $elm$svg$Svg$Attributes$viewBox = _VirtualDom_attribute('viewBox');
var $elm$svg$Svg$Attributes$width = _VirtualDom_attribute('width');
var $feathericons$elm_feather$FeatherIcons$toHtml = F2(
	function (attributes, _v0) {
		var src = _v0.a.src;
		var attrs = _v0.a.attrs;
		var strSize = $elm$core$String$fromFloat(attrs.size);
		var baseAttributes = _List_fromArray(
			[
				$elm$svg$Svg$Attributes$fill('none'),
				$elm$svg$Svg$Attributes$height(
				_Utils_ap(strSize, attrs.sizeUnit)),
				$elm$svg$Svg$Attributes$width(
				_Utils_ap(strSize, attrs.sizeUnit)),
				$elm$svg$Svg$Attributes$stroke('currentColor'),
				$elm$svg$Svg$Attributes$strokeLinecap('round'),
				$elm$svg$Svg$Attributes$strokeLinejoin('round'),
				$elm$svg$Svg$Attributes$strokeWidth(
				$elm$core$String$fromFloat(attrs.strokeWidth)),
				$elm$svg$Svg$Attributes$viewBox(attrs.viewBox)
			]);
		var combinedAttributes = _Utils_ap(
			function () {
				var _v1 = attrs._class;
				if (_v1.$ === 'Just') {
					var c = _v1.a;
					return A2(
						$elm$core$List$cons,
						$elm$svg$Svg$Attributes$class(c),
						baseAttributes);
				} else {
					return baseAttributes;
				}
			}(),
			attributes);
		return A2(
			$elm$svg$Svg$svg,
			combinedAttributes,
			A2(
				$elm$core$List$map,
				$elm$svg$Svg$map($elm$core$Basics$never),
				src));
	});
var $elm$svg$Svg$Attributes$points = _VirtualDom_attribute('points');
var $elm$svg$Svg$polyline = $elm$svg$Svg$trustedNode('polyline');
var $feathericons$elm_feather$FeatherIcons$trash2 = A2(
	$feathericons$elm_feather$FeatherIcons$makeBuilder,
	'trash-2',
	_List_fromArray(
		[
			A2(
			$elm$svg$Svg$polyline,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$points('3 6 5 6 21 6')
				]),
			_List_Nil),
			A2(
			$elm$svg$Svg$path,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$d('M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2')
				]),
			_List_Nil),
			A2(
			$elm$svg$Svg$line,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$x1('10'),
					$elm$svg$Svg$Attributes$y1('11'),
					$elm$svg$Svg$Attributes$x2('10'),
					$elm$svg$Svg$Attributes$y2('17')
				]),
			_List_Nil),
			A2(
			$elm$svg$Svg$line,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$x1('14'),
					$elm$svg$Svg$Attributes$y1('11'),
					$elm$svg$Svg$Attributes$x2('14'),
					$elm$svg$Svg$Attributes$y2('17')
				]),
			_List_Nil)
		]));
var $elm$core$Maybe$withDefault = F2(
	function (_default, maybe) {
		if (maybe.$ === 'Just') {
			var value = maybe.a;
			return value;
		} else {
			return _default;
		}
	});
var $author$project$Events$viewEvent = function (event) {
	if (event.$ === 'Exists') {
		var id = event.a.id;
		var name = event.a.name;
		var budget = event.a.budget;
		var comment = event.a.comment;
		return _List_fromArray(
			[
				A2(
				$elm$html$Html$span,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text(
						$elm$core$String$fromInt(id))
					])),
				A2(
				$elm$html$Html$span,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text(name)
					])),
				A2(
				$elm$html$Html$span,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text(budget)
					])),
				A2(
				$elm$html$Html$span,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text(
						A2($elm$core$Maybe$withDefault, '', comment))
					])),
				A2(
				$elm$html$Html$a,
				_List_fromArray(
					[
						$elm$html$Html$Attributes$href('#'),
						$elm$html$Html$Events$onClick(
						$author$project$Events$OpenModal(
							$author$project$Events$modalFromEvent(event)))
					]),
				_List_fromArray(
					[
						A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$edit)
					])),
				A2(
				$elm$html$Html$a,
				_List_fromArray(
					[
						$elm$html$Html$Attributes$href('#'),
						$elm$html$Html$Events$onClick(
						$author$project$Events$DeleteEvent(id))
					]),
				_List_fromArray(
					[
						A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$trash2)
					]))
			]);
	} else {
		return _List_fromArray(
			[
				A2(
				$elm$html$Html$span,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text('')
					])),
				A2(
				$elm$html$Html$span,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text('')
					])),
				A2(
				$elm$html$Html$span,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text('')
					])),
				A2(
				$elm$html$Html$span,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text('')
					])),
				A2(
				$elm$html$Html$a,
				_List_fromArray(
					[
						$elm$html$Html$Attributes$href('#'),
						$elm$html$Html$Events$onClick(
						$author$project$Events$OpenModal(
							$author$project$Events$modalFromEvent(event)))
					]),
				_List_fromArray(
					[
						A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$plus)
					])),
				A2($elm$html$Html$span, _List_Nil, _List_Nil)
			]);
	}
};
var $author$project$Events$newEventsList = function (webData) {
	return A2(
		$author$project$Utils$Main$mapWebdata,
		function (list) {
			return A4(
				$author$project$SearchList$new,
				$author$project$Events$EventListMsg,
				$author$project$Utils$Main$propertyFilter($author$project$Events$eventName),
				$author$project$Events$viewEvent,
				list);
		},
		webData);
};
var $author$project$Events$EventDetails = F2(
	function (a, b) {
		return {$: 'EventDetails', a: a, b: b};
	});
var $author$project$Events$MealModification = function (a) {
	return {$: 'MealModification', a: a};
};
var $author$project$Events$MealSearchMsg = function (a) {
	return {$: 'MealSearchMsg', a: a};
};
var $author$project$Events$mealName = function (meal) {
	if (meal.$ === 'Meal') {
		var recipe_name = meal.a.recipe_name;
		return recipe_name;
	} else {
		return '';
	}
};
var $author$project$Events$AddNewMeal = {$: 'AddNewMeal'};
var $author$project$Events$DeleteMeal = function (a) {
	return {$: 'DeleteMeal', a: a};
};
var $author$project$Events$EditMeal = function (a) {
	return {$: 'EditMeal', a: a};
};
var $author$project$Events$viewMeal = F2(
	function (event, meal) {
		if (meal.$ === 'Meal') {
			var recipe_name = meal.a.recipe_name;
			var place_name = meal.a.place_name;
			var start_time = meal.a.start_time;
			var price = meal.a.price;
			var weight = meal.a.weight;
			var servings = meal.a.servings;
			return _List_fromArray(
				[
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text(recipe_name)
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text(place_name)
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text(start_time)
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text(price + '€')
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text(
							$elm$core$String$fromFloat(weight) + 'kg')
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text(
							$elm$core$String$fromInt(servings))
						])),
					A2(
					$elm$html$Html$a,
					_List_fromArray(
						[
							$elm$html$Html$Attributes$href('#'),
							$elm$html$Html$Events$onClick(
							A2(
								$author$project$Events$EventDetails,
								event,
								$author$project$Events$EditMeal(meal)))
						]),
					_List_fromArray(
						[
							A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$edit)
						])),
					A2(
					$elm$html$Html$a,
					_List_fromArray(
						[
							$elm$html$Html$Attributes$href('#'),
							$elm$html$Html$Events$onClick(
							A2(
								$author$project$Events$EventDetails,
								event,
								$author$project$Events$DeleteMeal(meal)))
						]),
					_List_fromArray(
						[
							A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$trash2)
						]))
				]);
		} else {
			return _List_fromArray(
				[
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('')
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('')
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('')
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('')
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('')
						])),
					A2(
					$elm$html$Html$span,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('')
						])),
					A2(
					$elm$html$Html$a,
					_List_fromArray(
						[
							$elm$html$Html$Attributes$href('#'),
							$elm$html$Html$Events$onClick(
							A2(
								$author$project$Events$EventDetails,
								event,
								$author$project$Events$MealModification($author$project$Events$AddNewMeal)))
						]),
					_List_fromArray(
						[
							A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$edit)
						]))
				]);
		}
	});
var $author$project$Events$newMealsList = F2(
	function (event, webData) {
		return A2(
			$author$project$Utils$Main$mapWebdata,
			function (list) {
				return A4(
					$author$project$SearchList$new,
					function (msg) {
						return A2(
							$author$project$Events$EventDetails,
							event,
							$author$project$Events$MealModification(
								$author$project$Events$MealSearchMsg(msg)));
					},
					$author$project$Utils$Main$propertyFilter($author$project$Events$mealName),
					$author$project$Events$viewMeal(event),
					list);
			},
			webData);
	});
var $author$project$Utils$Main$toWebdata = function (r) {
	if (r.$ === 'Ok') {
		var a = r.a;
		return $author$project$Utils$Model$Success(a);
	} else {
		var e = r.a;
		return $author$project$Utils$Model$Failure(e);
	}
};
var $author$project$Events$handleWebDataMsg = F2(
	function (msg, data) {
		var eventModal = data.a.eventModal;
		var events = data.a.events;
		if (msg.$ === 'EventList') {
			if (msg.a.$ === 'Ok') {
				var list = msg.a.a;
				return _Utils_Tuple2(
					$author$project$Events$Data(
						{
							eventModal: eventModal,
							events: $author$project$Events$newEventsList(
								$author$project$Utils$Model$Success(
									_Utils_ap(
										list,
										_List_fromArray(
											[
												$author$project$Events$NewEvent(
												{budget: '', comment: $elm$core$Maybe$Nothing, name: ''})
											]))))
						}),
					$elm$core$Platform$Cmd$none);
			} else {
				var e = msg.a.a;
				return _Utils_Tuple2(
					$author$project$Events$Data(
						{
							eventModal: eventModal,
							events: $author$project$Utils$Model$Failure(e)
						}),
					$elm$core$Platform$Cmd$none);
			}
		} else {
			var list = msg.a;
			var setdetails = function () {
				if (eventModal.$ === 'Just') {
					var event = eventModal.a.a.event;
					return $elm$core$Maybe$Just(
						$author$project$Events$Details(
							{
								details: A2(
									$author$project$Events$newMealsList,
									event,
									$author$project$Utils$Main$toWebdata(list)),
								event: event,
								mealModal: $elm$core$Maybe$Nothing
							}));
				} else {
					return eventModal;
				}
			}();
			return _Utils_Tuple2(
				$author$project$Events$Data(
					{eventModal: setdetails, events: events}),
				$elm$core$Platform$Cmd$none);
		}
	});
var $author$project$Events$handleEventTabMsg = F2(
	function (msg, data) {
		var events = data.a.events;
		var eventModal = data.a.eventModal;
		switch (msg.$) {
			case 'EventListMsg':
				var searchListMsg = msg.a;
				if (events.$ === 'Success') {
					var searchList = events.a;
					var _v3 = A2($author$project$SearchList$handleMsg, searchList, searchListMsg);
					var superCmd = _v3.a;
					var newSearchList = _v3.b;
					var cmd = _v3.c;
					return _Utils_Tuple2(
						$author$project$Events$Data(
							{
								eventModal: eventModal,
								events: $author$project$Utils$Model$Success(newSearchList)
							}),
						$elm$core$Platform$Cmd$batch(
							_List_fromArray(
								[
									A2($elm$core$Platform$Cmd$map, $author$project$Events$EventListMsg, cmd),
									superCmd
								])));
				} else {
					return _Utils_Tuple2(data, $elm$core$Platform$Cmd$none);
				}
			case 'OpenModal':
				var open = msg.a;
				var _v4 = function () {
					var event = open.a.event;
					var mealModal = open.a.mealModal;
					if (event.$ === 'Exists') {
						var id = event.a.id;
						return _Utils_Tuple2(
							$elm$core$Maybe$Just(open),
							$author$project$Events$fetchMeals(id));
					} else {
						return _Utils_Tuple2(
							$elm$core$Maybe$Just(
								$author$project$Events$Details(
									{
										details: A2(
											$author$project$Events$newMealsList,
											event,
											$author$project$Utils$Model$Success(_List_Nil)),
										event: event,
										mealModal: mealModal
									})),
							$elm$core$Platform$Cmd$none);
					}
				}();
				var openModal = _v4.a;
				var cmd = _v4.b;
				return _Utils_Tuple2(
					$author$project$Events$Data(
						{eventModal: openModal, events: events}),
					cmd);
			case 'CloseModal':
				return _Utils_Tuple2(
					$author$project$Events$Data(
						{eventModal: $elm$core$Maybe$Nothing, events: events}),
					$elm$core$Platform$Cmd$none);
			case 'GotWebData':
				var wdMsg = msg.a;
				return A2($author$project$Events$handleWebDataMsg, wdMsg, data);
			case 'InitTab':
				return _Utils_Tuple2(data, $author$project$Events$fetchEvents);
			case 'SaveModal':
				return _Debug_todo(
					'Events',
					{
						start: {line: 439, column: 21},
						end: {line: 439, column: 31}
					})('SaveModal');
			case 'EventDetails':
				var ev = msg.a;
				var evMsg = msg.b;
				if (eventModal.$ === 'Nothing') {
					return _Utils_Tuple2(data, $elm$core$Platform$Cmd$none);
				} else {
					var evDetails = eventModal.a;
					var _v8 = A2($author$project$Events$handleEventDetailsMsg, evDetails, evMsg);
					var details = _v8.a;
					var cmd = _v8.b;
					return _Utils_eq(
						ev,
						$author$project$Events$getEvent(evDetails)) ? _Utils_Tuple2(data, $elm$core$Platform$Cmd$none) : _Utils_Tuple2(
						$author$project$Events$Data(
							{
								eventModal: $elm$core$Maybe$Just(details),
								events: events
							}),
						cmd);
				}
			default:
				var id = msg.a;
				return _Utils_Tuple2(
					data,
					$author$project$Events$deleteEvent(id));
		}
	});
var $author$project$Ingredients$Model$Add = function (a) {
	return {$: 'Add', a: a};
};
var $author$project$Ingredients$Model$IngredientEditor = F4(
	function (id, name, energy, comment) {
		return {comment: comment, energy: energy, id: id, name: name};
	});
var $author$project$Ingredients$Model$Edit = function (a) {
	return {$: 'Edit', a: a};
};
var $elm$core$List$filter = F2(
	function (isGood, list) {
		return A3(
			$elm$core$List$foldr,
			F2(
				function (x, xs) {
					return isGood(x) ? A2($elm$core$List$cons, x, xs) : xs;
				}),
			_List_Nil,
			list);
	});
var $elm$core$List$head = function (list) {
	if (list.b) {
		var x = list.a;
		var xs = list.b;
		return $elm$core$Maybe$Just(x);
	} else {
		return $elm$core$Maybe$Nothing;
	}
};
var $elm$core$Maybe$map = F2(
	function (f, maybe) {
		if (maybe.$ === 'Just') {
			var value = maybe.a;
			return $elm$core$Maybe$Just(
				f(value));
		} else {
			return $elm$core$Maybe$Nothing;
		}
	});
var $author$project$Ingredients$Update$editor = F2(
	function (itab, id) {
		var _v0 = itab.ingredients;
		if (_v0.$ === 'Success') {
			var ingredients = _v0.a;
			return A2(
				$elm$core$Maybe$withDefault,
				itab.modal,
				A2(
					$elm$core$Maybe$map,
					function (i) {
						return $author$project$Ingredients$Model$Edit(
							A4(
								$author$project$Ingredients$Model$IngredientEditor,
								$elm$core$Maybe$Just(i.id),
								i.name,
								$elm$core$String$fromFloat(i.energy),
								A2($elm$core$Maybe$withDefault, '', i.comment)));
					},
					$elm$core$List$head(
						A2(
							$elm$core$List$filter,
							function (i) {
								return _Utils_eq(i.id, id);
							},
							ingredients))));
		} else {
			return itab.modal;
		}
	});
var $author$project$Ingredients$Model$GotWebData = function (a) {
	return {$: 'GotWebData', a: a};
};
var $author$project$Ingredients$Model$IngredientsList = function (a) {
	return {$: 'IngredientsList', a: a};
};
var $author$project$Ingredients$Model$Ingredient = F4(
	function (id, name, energy, comment) {
		return {comment: comment, energy: energy, id: id, name: name};
	});
var $elm$json$Json$Decode$andThen = _Json_andThen;
var $elm$json$Json$Decode$fail = _Json_fail;
var $elm$core$String$toFloat = _String_toFloat;
var $author$project$Utils$Decoding$decodeStringFloat = function () {
	var parseFloat = function (s) {
		return A2(
			$elm$core$Maybe$withDefault,
			$elm$json$Json$Decode$fail('Could not parse float'),
			A2(
				$elm$core$Maybe$map,
				$elm$json$Json$Decode$succeed,
				$elm$core$String$toFloat(s)));
	};
	return A2($elm$json$Json$Decode$andThen, parseFloat, $elm$json$Json$Decode$string);
}();
var $elm$json$Json$Decode$null = _Json_decodeNull;
var $elm$json$Json$Decode$nullable = function (decoder) {
	return $elm$json$Json$Decode$oneOf(
		_List_fromArray(
			[
				$elm$json$Json$Decode$null($elm$core$Maybe$Nothing),
				A2($elm$json$Json$Decode$map, $elm$core$Maybe$Just, decoder)
			]));
};
var $author$project$Ingredients$Service$decodeIngredient = A5(
	$elm$json$Json$Decode$map4,
	$author$project$Ingredients$Model$Ingredient,
	A2($elm$json$Json$Decode$field, 'ingredient_id', $elm$json$Json$Decode$int),
	A2($elm$json$Json$Decode$field, 'name', $elm$json$Json$Decode$string),
	A2($elm$json$Json$Decode$field, 'energy', $author$project$Utils$Decoding$decodeStringFloat),
	A2(
		$elm$json$Json$Decode$field,
		'comment',
		$elm$json$Json$Decode$nullable($elm$json$Json$Decode$string)));
var $author$project$Ingredients$Service$decodeIngredientList = $elm$json$Json$Decode$list($author$project$Ingredients$Service$decodeIngredient);
var $author$project$Ingredients$Service$fetchIngredients = $elm$http$Http$get(
	{
		expect: A2(
			$elm$http$Http$expectJson,
			A2($elm$core$Basics$composeL, $author$project$Ingredients$Model$GotWebData, $author$project$Ingredients$Model$IngredientsList),
			$author$project$Ingredients$Service$decodeIngredientList),
		url: $author$project$Settings$backend('/ingredients/list')
	});
var $author$project$Ingredients$Model$CloseModal = {$: 'CloseModal'};
var $author$project$Ingredients$Model$SuccessfulPost = function (a) {
	return {$: 'SuccessfulPost', a: a};
};
var $elm$json$Json$Encode$int = _Json_wrap;
var $elm$json$Json$Encode$null = _Json_encodeNull;
var $elm$json$Json$Encode$object = function (pairs) {
	return _Json_wrap(
		A3(
			$elm$core$List$foldl,
			F2(
				function (_v0, obj) {
					var k = _v0.a;
					var v = _v0.b;
					return A3(_Json_addField, k, v, obj);
				}),
			_Json_emptyObject(_Utils_Tuple0),
			pairs));
};
var $author$project$Ingredients$Service$encodeIngredient = function (ingredient) {
	return $elm$json$Json$Encode$object(
		_List_fromArray(
			[
				_Utils_Tuple2(
				'ingredient_id',
				A2(
					$elm$core$Maybe$withDefault,
					$elm$json$Json$Encode$null,
					A2($elm$core$Maybe$map, $elm$json$Json$Encode$int, ingredient.id))),
				_Utils_Tuple2(
				'name',
				$elm$json$Json$Encode$string(ingredient.name)),
				_Utils_Tuple2(
				'energy',
				$elm$json$Json$Encode$string(ingredient.energy)),
				_Utils_Tuple2(
				'comment',
				$elm$json$Json$Encode$string(ingredient.comment))
			]));
};
var $elm$http$Http$jsonBody = function (value) {
	return A2(
		_Http_pair,
		'application/json',
		A2($elm$json$Json$Encode$encode, 0, value));
};
var $elm$http$Http$post = function (r) {
	return $elm$http$Http$request(
		{body: r.body, expect: r.expect, headers: _List_Nil, method: 'POST', timeout: $elm$core$Maybe$Nothing, tracker: $elm$core$Maybe$Nothing, url: r.url});
};
var $author$project$Ingredients$Service$addOrUpdateIngredient = function (ingredient) {
	var url = function () {
		var _v0 = ingredient.id;
		if (_v0.$ === 'Just') {
			var id = _v0.a;
			return '/ingredients/update/' + $elm$core$String$fromInt(id);
		} else {
			return '/ingredients/create';
		}
	}();
	return $elm$http$Http$post(
		{
			body: $elm$http$Http$jsonBody(
				$author$project$Ingredients$Service$encodeIngredient(ingredient)),
			expect: A2(
				$elm$http$Http$expectJson,
				A2($elm$core$Basics$composeL, $author$project$Ingredients$Model$GotWebData, $author$project$Ingredients$Model$SuccessfulPost),
				$elm$json$Json$Decode$int),
			url: $author$project$Settings$backend(url)
		});
};
var $author$project$Ingredients$Update$mapTab = F2(
	function (f, tab) {
		if (tab.$ === 'Ingredients') {
			var i = tab.a;
			return f(i);
		} else {
			var any = tab;
			return any;
		}
	});
var $author$project$Utils$Cursor$modifyAt = F3(
	function (index, f, cursor) {
		var mapper = F3(
			function (m, i, a) {
				return _Utils_eq(i, m) ? f(a) : a;
			});
		var lenRight = $elm$core$List$length(cursor.right);
		var lenLeft = $elm$core$List$length(cursor.left);
		var len = (lenLeft + lenRight) + 1;
		return ((index < 0) || (_Utils_cmp(index, len) > 0)) ? cursor : ((_Utils_cmp(index, lenLeft) < 0) ? _Utils_update(
			cursor,
			{
				left: A2(
					$elm$core$List$indexedMap,
					mapper(index),
					cursor.left)
			}) : (_Utils_eq(index, lenLeft) ? _Utils_update(
			cursor,
			{
				active: f(cursor.active)
			}) : _Utils_update(
			cursor,
			{
				right: A2(
					$elm$core$List$indexedMap,
					mapper((index - lenLeft) - 1),
					cursor.right)
			})));
	});
var $author$project$Ingredients$Update$updateModel = F2(
	function (f, model) {
		return _Utils_update(
			model,
			{
				tabs: A3($author$project$Utils$Cursor$modifyAt, 0, f, model.tabs)
			});
	});
var $author$project$Ingredients$Update$handleModalMsg = F2(
	function (msg, model) {
		var update = F2(
			function (modal, f) {
				switch (modal.$) {
					case 'Edit':
						var e = modal.a;
						return $author$project$Ingredients$Model$Edit(
							f(e));
					case 'Add':
						var e = modal.a;
						return $author$project$Ingredients$Model$Add(
							f(e));
					default:
						var any = modal;
						return any;
				}
			});
		var mapUpdate = function (f) {
			return $author$project$Ingredients$Update$mapTab(
				function (i) {
					return $author$project$Model$Ingredients(
						_Utils_update(
							i,
							{
								modal: A2(update, i.modal, f)
							}));
				});
		};
		switch (msg.$) {
			case 'EditName':
				var name = msg.a;
				return _Utils_Tuple2(
					A2(
						$author$project$Ingredients$Update$updateModel,
						mapUpdate(
							function (e) {
								return _Utils_update(
									e,
									{name: name});
							}),
						model),
					$elm$core$Platform$Cmd$none);
			case 'EditEnergy':
				var energy = msg.a;
				return _Utils_Tuple2(
					A2(
						$author$project$Ingredients$Update$updateModel,
						mapUpdate(
							function (e) {
								return _Utils_update(
									e,
									{energy: energy});
							}),
						model),
					$elm$core$Platform$Cmd$none);
			case 'EditComment':
				var comment = msg.a;
				return _Utils_Tuple2(
					A2(
						$author$project$Ingredients$Update$updateModel,
						mapUpdate(
							function (e) {
								return _Utils_update(
									e,
									{comment: comment});
							}),
						model),
					$elm$core$Platform$Cmd$none);
			default:
				var e = msg.a;
				var save = $author$project$Ingredients$Update$mapTab(
					function (i) {
						return $author$project$Model$Ingredients(
							_Utils_update(
								i,
								{modal: $author$project$Ingredients$Model$NoModal}));
					});
				return _Utils_Tuple2(
					A2($author$project$Ingredients$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$batch(
						_List_fromArray(
							[
								A2(
								$elm$core$Platform$Cmd$map,
								$author$project$Model$IngredientMessage,
								$author$project$Ingredients$Service$addOrUpdateIngredient(e)),
								A2(
								$elm$core$Platform$Cmd$map,
								function (_v1) {
									return $author$project$Model$IngredientMessage($author$project$Ingredients$Model$CloseModal);
								},
								$elm$core$Platform$Cmd$none)
							])));
		}
	});
var $author$project$Ingredients$Update$handleWebData = F2(
	function (data, model) {
		if (data.$ === 'IngredientsList') {
			var ingredients = data.a;
			var save = $author$project$Ingredients$Update$mapTab(
				function (i) {
					return $author$project$Model$Ingredients(
						_Utils_update(
							i,
							{
								ingredients: $author$project$Utils$Main$toWebdata(ingredients)
							}));
				});
			return _Utils_Tuple2(
				A2($author$project$Ingredients$Update$updateModel, save, model),
				$elm$core$Platform$Cmd$none);
		} else {
			var id = data.a;
			return _Utils_Tuple2(
				model,
				A2($elm$core$Platform$Cmd$map, $author$project$Model$IngredientMessage, $author$project$Ingredients$Service$fetchIngredients));
		}
	});
var $author$project$Ingredients$Update$handleMsg = F2(
	function (msg, model) {
		switch (msg.$) {
			case 'GotWebData':
				var data = msg.a;
				return A2($author$project$Ingredients$Update$handleWebData, data, model);
			case 'EditFilter':
				var s = msg.a;
				var save = $author$project$Ingredients$Update$mapTab(
					function (i) {
						return $author$project$Model$Ingredients(
							_Utils_update(
								i,
								{filter: s}));
					});
				return _Utils_Tuple2(
					A2($author$project$Ingredients$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'AddIngredient':
				var save = $author$project$Ingredients$Update$mapTab(
					function (i) {
						return $author$project$Model$Ingredients(
							_Utils_update(
								i,
								{
									modal: $author$project$Ingredients$Model$Add(
										A4($author$project$Ingredients$Model$IngredientEditor, $elm$core$Maybe$Nothing, '', '', ''))
								}));
					});
				return _Utils_Tuple2(
					A2($author$project$Ingredients$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'EditIngredient':
				var id = msg.a;
				var save = $author$project$Ingredients$Update$mapTab(
					function (i) {
						return $author$project$Model$Ingredients(
							_Utils_update(
								i,
								{
									modal: A2($author$project$Ingredients$Update$editor, i, id)
								}));
					});
				return _Utils_Tuple2(
					A2($author$project$Ingredients$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'CloseModal':
				var save = $author$project$Ingredients$Update$mapTab(
					function (i) {
						return $author$project$Model$Ingredients(
							_Utils_update(
								i,
								{modal: $author$project$Ingredients$Model$NoModal}));
					});
				return _Utils_Tuple2(
					A2($author$project$Ingredients$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'ModalMsg':
				var m = msg.a;
				return A2($author$project$Ingredients$Update$handleModalMsg, m, model);
			case 'DeleteIngredient':
				var id = msg.a;
				return _Debug_todo(
					'Ingredients.Update',
					{
						start: {line: 78, column: 13},
						end: {line: 78, column: 23}
					})('Delete ingredient');
			default:
				var save = $author$project$Ingredients$Update$mapTab(
					function (i) {
						return $author$project$Model$Ingredients(
							_Utils_update(
								i,
								{ingredients: $author$project$Utils$Model$Loading}));
					});
				return _Utils_Tuple2(
					A2($author$project$Ingredients$Update$updateModel, save, model),
					A2($elm$core$Platform$Cmd$map, $author$project$Model$IngredientMessage, $author$project$Ingredients$Service$fetchIngredients));
		}
	});
var $author$project$Ingredients$Main$handleIngredientsMsg = $author$project$Ingredients$Update$handleMsg;
var $author$project$Recipes$Model$Add = function (a) {
	return {$: 'Add', a: a};
};
var $author$project$Recipes$Model$Edit = function (a) {
	return {$: 'Edit', a: a};
};
var $author$project$Recipes$Model$GotWebData = function (a) {
	return {$: 'GotWebData', a: a};
};
var $author$project$Recipes$Model$RecipeId = F2(
	function (a, b) {
		return {$: 'RecipeId', a: a, b: b};
	});
var $author$project$Utils$Decoding$maybe = F2(
	function (f, m) {
		if (m.$ === 'Just') {
			var value = m.a;
			return f(value);
		} else {
			return $elm$json$Json$Encode$null;
		}
	});
var $author$project$Recipes$Service$encodeRecipeEditor = function (editor) {
	return $elm$json$Json$Encode$object(
		_List_fromArray(
			[
				_Utils_Tuple2(
				'recipe_id',
				A2($author$project$Utils$Decoding$maybe, $elm$json$Json$Encode$int, editor.id)),
				_Utils_Tuple2(
				'name',
				$elm$json$Json$Encode$string(editor.name)),
				_Utils_Tuple2(
				'comment',
				A2($author$project$Utils$Decoding$maybe, $elm$json$Json$Encode$string, editor.comment))
			]));
};
var $author$project$Recipes$Service$updateRecipeEditor = F2(
	function (url, editor) {
		return $elm$http$Http$post(
			{
				body: $elm$http$Http$jsonBody(
					$author$project$Recipes$Service$encodeRecipeEditor(editor)),
				expect: A2(
					$elm$http$Http$expectJson,
					A2(
						$elm$core$Basics$composeL,
						$author$project$Recipes$Model$GotWebData,
						$author$project$Recipes$Model$RecipeId(editor)),
					$elm$json$Json$Decode$int),
				url: 'http://localhost:3000/recipes' + url
			});
	});
var $author$project$Recipes$Service$addOrUpdateRecipe = function (modal) {
	switch (modal.$) {
		case 'Add':
			var editor = modal.a;
			return A2($author$project$Recipes$Service$updateRecipeEditor, '/create', editor);
		case 'Edit':
			var editor = modal.a;
			var _v1 = editor.id;
			if (_v1.$ === 'Just') {
				var id = _v1.a;
				return A2(
					$author$project$Recipes$Service$updateRecipeEditor,
					'/' + ($elm$core$String$fromInt(id) + '/update'),
					editor);
			} else {
				return $elm$core$Platform$Cmd$none;
			}
		default:
			return $elm$core$Platform$Cmd$none;
	}
};
var $elm$core$Maybe$andThen = F2(
	function (callback, maybeValue) {
		if (maybeValue.$ === 'Just') {
			var value = maybeValue.a;
			return callback(value);
		} else {
			return $elm$core$Maybe$Nothing;
		}
	});
var $author$project$Recipes$Model$editorFromReipe = function (recipe) {
	return {
		comment: recipe.comment,
		id: $elm$core$Maybe$Just(recipe.id),
		ingredients: $author$project$Utils$Model$NotAsked,
		name: recipe.name,
		steps: $author$project$Utils$Model$NotAsked
	};
};
var $author$project$Recipes$Model$emptyRecipeEditor = {
	comment: $elm$core$Maybe$Nothing,
	id: $elm$core$Maybe$Nothing,
	ingredients: $author$project$Utils$Model$Success(_List_Nil),
	name: '',
	steps: $author$project$Utils$Model$Success(_List_Nil)
};
var $author$project$Recipes$Model$MetaIngredientData = function (a) {
	return {$: 'MetaIngredientData', a: a};
};
var $author$project$Recipes$Model$IsDirect = function (a) {
	return {$: 'IsDirect', a: a};
};
var $author$project$Recipes$Model$IsSubRecipe = function (a) {
	return {$: 'IsSubRecipe', a: a};
};
var $author$project$Recipes$Model$Recipe = F3(
	function (id, name, comment) {
		return {comment: comment, id: id, name: name};
	});
var $elm$json$Json$Decode$map3 = _Json_map3;
var $author$project$Recipes$Service$decodeRecipe = A4(
	$elm$json$Json$Decode$map3,
	$author$project$Recipes$Model$Recipe,
	A2($elm$json$Json$Decode$field, 'recipe_id', $elm$json$Json$Decode$int),
	A2($elm$json$Json$Decode$field, 'name', $elm$json$Json$Decode$string),
	A2(
		$elm$json$Json$Decode$field,
		'comment',
		$elm$json$Json$Decode$nullable($elm$json$Json$Decode$string)));
var $author$project$Recipes$Service$decodeMetaIngredient = $elm$json$Json$Decode$oneOf(
	_List_fromArray(
		[
			A2(
			$elm$json$Json$Decode$map,
			$author$project$Recipes$Model$IsSubRecipe,
			A2($elm$json$Json$Decode$field, 'MetaRecipe', $author$project$Recipes$Service$decodeRecipe)),
			A2(
			$elm$json$Json$Decode$map,
			$author$project$Recipes$Model$IsDirect,
			A2($elm$json$Json$Decode$field, 'Ingredient', $author$project$Ingredients$Service$decodeIngredient))
		]));
var $author$project$Recipes$Service$decodeMetaIngredients = $elm$json$Json$Decode$list($author$project$Recipes$Service$decodeMetaIngredient);
var $author$project$Recipes$Service$fetchAllMetaIngredients = $elm$http$Http$get(
	{
		expect: A2(
			$elm$http$Http$expectJson,
			A2($elm$core$Basics$composeL, $author$project$Recipes$Model$GotWebData, $author$project$Recipes$Model$MetaIngredientData),
			$author$project$Recipes$Service$decodeMetaIngredients),
		url: 'http://localhost:3000/recipes/meta_ingredients/list'
	});
var $author$project$Recipes$Model$RecipeIngredientData = function (a) {
	return {$: 'RecipeIngredientData', a: a};
};
var $author$project$Recipes$Model$WeightedMetaIngredient = F3(
	function (metaIngredient, amount, unit) {
		return {amount: amount, metaIngredient: metaIngredient, unit: unit};
	});
var $author$project$Utils$Model$Unit = F2(
	function (unit_id, name) {
		return {name: name, unit_id: unit_id};
	});
var $author$project$Utils$Decoding$decodeUnit = A3(
	$elm$json$Json$Decode$map2,
	$author$project$Utils$Model$Unit,
	A2($elm$json$Json$Decode$field, 'unit_id', $elm$json$Json$Decode$int),
	A2($elm$json$Json$Decode$field, 'name', $elm$json$Json$Decode$string));
var $author$project$Recipes$Service$decodeNestedWeightedMetaIngredient = A4(
	$elm$json$Json$Decode$map3,
	$author$project$Recipes$Model$WeightedMetaIngredient,
	A2($elm$json$Json$Decode$field, 'ingredient', $author$project$Recipes$Service$decodeMetaIngredient),
	A2($elm$json$Json$Decode$field, 'amount', $elm$json$Json$Decode$string),
	A2($elm$json$Json$Decode$field, 'unit', $author$project$Utils$Decoding$decodeUnit));
var $author$project$Recipes$Service$decodeNestedWeightedMetaIngredients = $elm$json$Json$Decode$list($author$project$Recipes$Service$decodeNestedWeightedMetaIngredient);
var $author$project$Recipes$Service$fetchRecipeIngredients = function (recipeId) {
	return $elm$http$Http$get(
		{
			expect: A2(
				$elm$http$Http$expectJson,
				A2($elm$core$Basics$composeL, $author$project$Recipes$Model$GotWebData, $author$project$Recipes$Model$RecipeIngredientData),
				$author$project$Recipes$Service$decodeNestedWeightedMetaIngredients),
			url: 'http://localhost:3000/recipes/' + ($elm$core$String$fromInt(recipeId) + '/meta_ingredients/list')
		});
};
var $author$project$Recipes$Model$RecipesData = function (a) {
	return {$: 'RecipesData', a: a};
};
var $author$project$Recipes$Service$decodeRecipes = $elm$json$Json$Decode$list($author$project$Recipes$Service$decodeRecipe);
var $author$project$Recipes$Service$fetchRecipes = $elm$http$Http$get(
	{
		expect: A2(
			$elm$http$Http$expectJson,
			A2($elm$core$Basics$composeL, $author$project$Recipes$Model$GotWebData, $author$project$Recipes$Model$RecipesData),
			$author$project$Recipes$Service$decodeRecipes),
		url: 'http://localhost:3000/recipes/list'
	});
var $author$project$Recipes$Model$UnitData = function (a) {
	return {$: 'UnitData', a: a};
};
var $author$project$Recipes$Service$fetchUnits = $elm$http$Http$get(
	{
		expect: A2(
			$elm$http$Http$expectJson,
			A2($elm$core$Basics$composeL, $author$project$Recipes$Model$GotWebData, $author$project$Recipes$Model$UnitData),
			$elm$json$Json$Decode$list($author$project$Utils$Decoding$decodeUnit)),
		url: 'http://localhost:3000/utils/units'
	});
var $author$project$Recipes$Model$IngredientId = function (a) {
	return {$: 'IngredientId', a: a};
};
var $author$project$Utils$Model$newDropdownData = function (selected) {
	return {filter: '', open: false, selected: selected};
};
var $author$project$Recipes$Model$buildEditor = function (ingredient) {
	return {
		amountInput: '',
		ingredientDropdown: $author$project$Utils$Model$newDropdownData(
			$elm$core$Maybe$Just(ingredient.metaIngredient)),
		unitDropdown: $author$project$Utils$Model$newDropdownData(
			$elm$core$Maybe$Just(ingredient.unit))
	};
};
var $elm$regex$Regex$Match = F4(
	function (match, index, number, submatches) {
		return {index: index, match: match, number: number, submatches: submatches};
	});
var $elm$regex$Regex$contains = _Regex_contains;
var $elm$regex$Regex$fromStringWith = _Regex_fromStringWith;
var $elm$regex$Regex$fromString = function (string) {
	return A2(
		$elm$regex$Regex$fromStringWith,
		{caseInsensitive: false, multiline: false},
		string);
};
var $elm$regex$Regex$never = _Regex_never;
var $author$project$Utils$Decoding$floatRegex = A2(
	$elm$core$Maybe$withDefault,
	$elm$regex$Regex$never,
	$elm$regex$Regex$fromString('^[0-9]+(\\.[0-9]+)?$'));
var $author$project$Recipes$Update$isId = F2(
	function (id, meta) {
		var _v0 = _Utils_Tuple2(meta.metaIngredient, id);
		_v0$2:
		while (true) {
			if (_v0.a.$ === 'IsDirect') {
				if (_v0.b.$ === 'IngredientId') {
					var ig = _v0.a.a;
					var i = _v0.b.a;
					return _Utils_eq(ig.id, i);
				} else {
					break _v0$2;
				}
			} else {
				if (_v0.b.$ === 'SubRecipeId') {
					var sr = _v0.a.a;
					var i = _v0.b.a;
					return _Utils_eq(sr.id, i);
				} else {
					break _v0$2;
				}
			}
		}
		return false;
	});
var $author$project$Recipes$Update$mapTab = F2(
	function (f, tab) {
		if (tab.$ === 'Recipes') {
			var r = tab.a;
			return f(r);
		} else {
			var any = tab;
			return any;
		}
	});
var $author$project$Recipes$Update$updateModal = F2(
	function (modal, f) {
		switch (modal.$) {
			case 'Edit':
				var e = modal.a;
				return $author$project$Recipes$Model$Edit(
					f(e));
			case 'Add':
				var e = modal.a;
				return $author$project$Recipes$Model$Add(
					f(e));
			default:
				var any = modal;
				return any;
		}
	});
var $author$project$Recipes$Update$mapModalUpdate = function (f) {
	return $author$project$Recipes$Update$mapTab(
		function (i) {
			return $author$project$Model$Recipes(
				_Utils_update(
					i,
					{
						modal: A2($author$project$Recipes$Update$updateModal, i.modal, f)
					}));
		});
};
var $elm$core$Basics$not = _Basics_not;
var $author$project$Recipes$Update$updateModel = F2(
	function (f, model) {
		return _Utils_update(
			model,
			{
				tabs: A3($author$project$Utils$Cursor$modifyAt, 1, f, model.tabs)
			});
	});
var $author$project$Recipes$Update$handleMetaIngredientMsg = F3(
	function (msg, id, model) {
		var _new = function (_v2) {
			var i = _v2.a;
			var e = _v2.b;
			var unitDropdown = e.unitDropdown;
			var ingredientDropdown = e.ingredientDropdown;
			switch (msg.$) {
				case 'SetIngredientFilter':
					var filter = msg.a;
					return _Utils_Tuple2(
						i,
						_Utils_update(
							e,
							{
								ingredientDropdown: _Utils_update(
									ingredientDropdown,
									{filter: filter})
							}));
				case 'SetUnitFilter':
					var filter = msg.a;
					return _Utils_Tuple2(
						i,
						_Utils_update(
							e,
							{
								unitDropdown: _Utils_update(
									unitDropdown,
									{filter: filter})
							}));
				case 'SetIngredient':
					var ingredient = msg.a;
					return _Utils_Tuple2(
						_Utils_update(
							i,
							{metaIngredient: ingredient}),
						_Utils_update(
							e,
							{
								ingredientDropdown: _Utils_update(
									ingredientDropdown,
									{
										open: false,
										selected: $elm$core$Maybe$Just(ingredient)
									})
							}));
				case 'SetUnit':
					var unit = msg.a;
					return _Utils_Tuple2(
						_Utils_update(
							i,
							{unit: unit}),
						_Utils_update(
							e,
							{
								unitDropdown: _Utils_update(
									unitDropdown,
									{
										open: false,
										selected: $elm$core$Maybe$Just(unit)
									})
							}));
				case 'SetAmount':
					var amount = msg.a;
					return A2($elm$regex$Regex$contains, $author$project$Utils$Decoding$floatRegex, amount) ? _Utils_Tuple2(
						_Utils_update(
							i,
							{amount: amount}),
						e) : _Utils_Tuple2(i, e);
				default:
					return _Utils_Tuple2(i, e);
			}
		};
		var mapIf = F2(
			function (check, f) {
				return $elm$core$List$map(
					function (i) {
						return check(i) ? f(i) : i;
					});
			});
		var save = function (f) {
			return $author$project$Recipes$Update$mapModalUpdate(
				function (e) {
					return _Utils_update(
						e,
						{
							ingredients: A2(
								$author$project$Utils$Main$mapWebdata,
								A2(
									mapIf,
									A2(
										$elm$core$Basics$composeL,
										$author$project$Recipes$Update$isId(id),
										$elm$core$Tuple$first),
									f),
								e.ingredients)
						});
				});
		};
		if (msg.$ === 'Delete') {
			return _Utils_Tuple2(
				A2(
					$author$project$Recipes$Update$updateModel,
					$author$project$Recipes$Update$mapModalUpdate(
						function (e) {
							return _Utils_update(
								e,
								{
									ingredients: A2(
										$author$project$Utils$Main$mapWebdata,
										$elm$core$List$filter(
											A2(
												$elm$core$Basics$composeL,
												A2(
													$elm$core$Basics$composeL,
													$elm$core$Basics$not,
													$author$project$Recipes$Update$isId(id)),
												$elm$core$Tuple$first)),
										e.ingredients)
								});
						}),
					model),
				$elm$core$Platform$Cmd$none);
		} else {
			return _Utils_Tuple2(
				A2(
					$author$project$Recipes$Update$updateModel,
					save(_new),
					model),
				$elm$core$Platform$Cmd$none);
		}
	});
var $author$project$Recipes$Update$handleStepMsg = F3(
	function (msg, id, model) {
		return _Debug_todo(
			'Recipes.Update',
			{
				start: {line: 295, column: 5},
				end: {line: 295, column: 15}
			})('handleStepMsg');
	});
var $elm$core$Basics$negate = function (n) {
	return -n;
};
var $author$project$Recipes$Update$handleModalMsg = F2(
	function (msg, model) {
		var defaultIngredient = A3(
			$author$project$Recipes$Model$WeightedMetaIngredient,
			$author$project$Recipes$Model$IsDirect(
				A4($author$project$Ingredients$Model$Ingredient, -1, '', 0, $elm$core$Maybe$Nothing)),
			'',
			A2($author$project$Utils$Model$Unit, 0, ''));
		var addEntry = function (entry) {
			return A2(
				$author$project$Recipes$Update$updateModel,
				$author$project$Recipes$Update$mapModalUpdate(
					function (e) {
						return _Utils_update(
							e,
							{
								ingredients: A2(
									$author$project$Utils$Main$mapWebdata,
									function (d) {
										return _Utils_ap(
											d,
											_List_fromArray(
												[entry]));
									},
									e.ingredients)
							});
					}),
				model);
		};
		switch (msg.$) {
			case 'EditComment':
				var comment = msg.a;
				return _Utils_Tuple2(
					A2(
						$author$project$Recipes$Update$updateModel,
						$author$project$Recipes$Update$mapModalUpdate(
							function (e) {
								return _Utils_update(
									e,
									{
										comment: $elm$core$Maybe$Just(comment)
									});
							}),
						model),
					$elm$core$Platform$Cmd$none);
			case 'EditName':
				var name = msg.a;
				return _Utils_Tuple2(
					A2(
						$author$project$Recipes$Update$updateModel,
						$author$project$Recipes$Update$mapModalUpdate(
							function (e) {
								return _Utils_update(
									e,
									{name: name});
							}),
						model),
					$elm$core$Platform$Cmd$none);
			case 'EditMetaIngredient':
				var id = msg.a;
				var recipeIngredientMsg = msg.b;
				return A3($author$project$Recipes$Update$handleMetaIngredientMsg, recipeIngredientMsg, id, model);
			case 'AddMetaIngredient':
				var recipeIngredientMsg = msg.a;
				return A3(
					$author$project$Recipes$Update$handleMetaIngredientMsg,
					recipeIngredientMsg,
					$author$project$Recipes$Model$IngredientId(-1),
					addEntry(
						_Utils_Tuple2(
							defaultIngredient,
							$author$project$Recipes$Model$buildEditor(defaultIngredient))));
			default:
				var stepMsg = msg.a;
				var id = msg.b;
				return A3($author$project$Recipes$Update$handleStepMsg, stepMsg, id, model);
		}
	});
var $author$project$Recipes$Model$PostResult = function (a) {
	return {$: 'PostResult', a: a};
};
var $author$project$Recipes$Service$encodeRecipe = function (recipe) {
	return $elm$json$Json$Encode$object(
		_List_fromArray(
			[
				_Utils_Tuple2(
				'recipe_id',
				$elm$json$Json$Encode$int(recipe.id)),
				_Utils_Tuple2(
				'name',
				$elm$json$Json$Encode$string(recipe.name)),
				_Utils_Tuple2(
				'comment',
				A2($author$project$Utils$Decoding$maybe, $elm$json$Json$Encode$string, recipe.comment))
			]));
};
var $elm$json$Json$Encode$float = _Json_wrap;
var $author$project$Recipes$Service$encodeMetaIngredient = function (ingredient) {
	if (ingredient.$ === 'IsSubRecipe') {
		var recipe = ingredient.a;
		return $elm$json$Json$Encode$object(
			_List_fromArray(
				[
					_Utils_Tuple2(
					'MetaRecipe',
					$author$project$Recipes$Service$encodeRecipe(recipe))
				]));
	} else {
		var i = ingredient.a;
		return $elm$json$Json$Encode$object(
			_List_fromArray(
				[
					_Utils_Tuple2(
					'Ingredient',
					$elm$json$Json$Encode$object(
						_List_fromArray(
							[
								_Utils_Tuple2(
								'ingredient_id',
								$elm$json$Json$Encode$int(i.id)),
								_Utils_Tuple2(
								'name',
								$elm$json$Json$Encode$string(i.name)),
								_Utils_Tuple2(
								'comment',
								A2($author$project$Utils$Decoding$maybe, $elm$json$Json$Encode$string, i.comment)),
								_Utils_Tuple2(
								'energy',
								$elm$json$Json$Encode$float(i.energy))
							])))
				]));
	}
};
var $author$project$Utils$Decoding$encodeUnit = function (unit) {
	return $elm$json$Json$Encode$object(
		_List_fromArray(
			[
				_Utils_Tuple2(
				'unit_id',
				$elm$json$Json$Encode$int(unit.unit_id)),
				_Utils_Tuple2(
				'name',
				$elm$json$Json$Encode$string(unit.name))
			]));
};
var $author$project$Recipes$Service$encodeWeightedMetaIngredient = function (ingredient) {
	return $elm$json$Json$Encode$object(
		_List_fromArray(
			[
				_Utils_Tuple2(
				'ingredient',
				$author$project$Recipes$Service$encodeMetaIngredient(ingredient.metaIngredient)),
				_Utils_Tuple2(
				'amount',
				$elm$json$Json$Encode$string(
					(ingredient.amount === '') ? '0' : ingredient.amount)),
				_Utils_Tuple2(
				'unit',
				$author$project$Utils$Decoding$encodeUnit(ingredient.unit))
			]));
};
var $elm$json$Json$Encode$list = F2(
	function (func, entries) {
		return _Json_wrap(
			A3(
				$elm$core$List$foldl,
				_Json_addEntry(func),
				_Json_emptyArray(_Utils_Tuple0),
				entries));
	});
var $author$project$Recipes$Service$encodeMetaIngredients = function (ingredients) {
	if (ingredients.$ === 'Success') {
		var i = ingredients.a;
		return A2(
			$elm$json$Json$Encode$list,
			$author$project$Recipes$Service$encodeWeightedMetaIngredient,
			A2($elm$core$List$map, $elm$core$Tuple$first, i));
	} else {
		return $elm$json$Json$Encode$null;
	}
};
var $author$project$Recipes$Service$updateRecipeIngredients = F2(
	function (editor, id) {
		return $elm$http$Http$post(
			{
				body: $elm$http$Http$jsonBody(
					$author$project$Recipes$Service$encodeMetaIngredients(editor.ingredients)),
				expect: A2(
					$elm$http$Http$expectJson,
					A2($elm$core$Basics$composeL, $author$project$Recipes$Model$GotWebData, $author$project$Recipes$Model$PostResult),
					$elm$json$Json$Decode$succeed(_Utils_Tuple0)),
				url: 'http://localhost:3000/recipes/' + ($elm$core$String$fromInt(id) + '/meta_ingredients/update')
			});
	});
var $author$project$Recipes$Service$encodeStep = function (step) {
	return $elm$json$Json$Encode$object(
		_List_fromArray(
			[
				_Utils_Tuple2(
				'step_id',
				A2($author$project$Utils$Decoding$maybe, $elm$json$Json$Encode$int, step.id)),
				_Utils_Tuple2(
				'title',
				$elm$json$Json$Encode$string(step.title)),
				_Utils_Tuple2(
				'description',
				$elm$json$Json$Encode$string(step.description)),
				_Utils_Tuple2(
				'order',
				$elm$json$Json$Encode$float(step.order))
			]));
};
var $author$project$Recipes$Service$encodeSteps = function (steps) {
	if (steps.$ === 'Success') {
		var s = steps.a;
		return A2($elm$json$Json$Encode$list, $author$project$Recipes$Service$encodeStep, s);
	} else {
		return $elm$json$Json$Encode$null;
	}
};
var $author$project$Recipes$Service$updateRecipeSteps = F2(
	function (editor, id) {
		return $elm$http$Http$post(
			{
				body: $elm$http$Http$jsonBody(
					$author$project$Recipes$Service$encodeSteps(editor.steps)),
				expect: A2(
					$elm$http$Http$expectJson,
					A2($elm$core$Basics$composeL, $author$project$Recipes$Model$GotWebData, $author$project$Recipes$Model$PostResult),
					$elm$json$Json$Decode$succeed(_Utils_Tuple0)),
				url: 'http://localhost:3000/recipes/' + ($elm$core$String$fromInt(id) + '/steps/update')
			});
	});
var $author$project$Recipes$Service$updateRecipeExtras = F2(
	function (editor, id) {
		return $elm$core$Platform$Cmd$batch(
			_List_fromArray(
				[
					A2($author$project$Recipes$Service$updateRecipeIngredients, editor, id),
					A2($author$project$Recipes$Service$updateRecipeSteps, editor, id)
				]));
	});
var $author$project$Recipes$Update$handleWebData = F2(
	function (result, model) {
		switch (result.$) {
			case 'RecipesData':
				var recipes = result.a;
				var save = $author$project$Recipes$Update$mapTab(
					function (r) {
						return $author$project$Model$Recipes(
							_Utils_update(
								r,
								{
									recipes: $author$project$Utils$Main$toWebdata(recipes)
								}));
					});
				return _Utils_Tuple2(
					A2($author$project$Recipes$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'MetaIngredientData':
				var meta = result.a;
				var save = $author$project$Recipes$Update$mapTab(
					function (r) {
						return $author$project$Model$Recipes(
							_Utils_update(
								r,
								{
									allIngredients: $author$project$Utils$Main$toWebdata(meta)
								}));
					});
				return _Utils_Tuple2(
					A2($author$project$Recipes$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'UnitData':
				var units = result.a;
				var save = $author$project$Recipes$Update$mapTab(
					function (r) {
						return $author$project$Model$Recipes(
							_Utils_update(
								r,
								{
									allUnits: $author$project$Utils$Main$toWebdata(units)
								}));
					});
				return _Utils_Tuple2(
					A2($author$project$Recipes$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'RecipeIngredientData':
				var meta = result.a;
				var wd = $author$project$Utils$Main$toWebdata(meta);
				if (wd.$ === 'Success') {
					var ingredients = wd.a;
					var newRecipeIngredients = A2(
						$elm$core$List$map,
						function (i) {
							return _Utils_Tuple2(
								i,
								$author$project$Recipes$Model$buildEditor(i));
						},
						ingredients);
					var save = $author$project$Recipes$Update$mapModalUpdate(
						function (e) {
							return _Utils_update(
								e,
								{
									ingredients: $author$project$Utils$Model$Success(newRecipeIngredients)
								});
						});
					return _Utils_Tuple2(
						A2($author$project$Recipes$Update$updateModel, save, model),
						$elm$core$Platform$Cmd$none);
				} else {
					return _Utils_Tuple2(model, $elm$core$Platform$Cmd$none);
				}
			case 'RecipeId':
				var editor = result.a;
				var meta = result.b;
				if (meta.$ === 'Ok') {
					var id = meta.a;
					return _Utils_Tuple2(
						model,
						A2(
							$elm$core$Platform$Cmd$map,
							$author$project$Model$RecipeMessage,
							A2($author$project$Recipes$Service$updateRecipeExtras, editor, id)));
				} else {
					return _Utils_Tuple2(model, $elm$core$Platform$Cmd$none);
				}
			default:
				return _Utils_Tuple2(model, $elm$core$Platform$Cmd$none);
		}
	});
var $author$project$Recipes$Update$recipeList = function (model) {
	var _v0 = model.tabs.active;
	if (_v0.$ === 'Recipes') {
		var r = _v0.a;
		var _v1 = r.recipes;
		if (_v1.$ === 'Success') {
			var recipes = _v1.a;
			return $elm$core$Maybe$Just(recipes);
		} else {
			return $elm$core$Maybe$Nothing;
		}
	} else {
		return $elm$core$Maybe$Nothing;
	}
};
var $author$project$Recipes$Update$handleMsg = F2(
	function (msg, model) {
		switch (msg.$) {
			case 'GotWebData':
				var data = msg.a;
				return A2($author$project$Recipes$Update$handleWebData, data, model);
			case 'InitTab':
				var save = $author$project$Recipes$Update$mapTab(
					function (r) {
						return $author$project$Model$Recipes(
							_Utils_update(
								r,
								{recipes: $author$project$Utils$Model$Loading}));
					});
				return _Utils_Tuple2(
					A2($author$project$Recipes$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$batch(
						_List_fromArray(
							[
								A2($elm$core$Platform$Cmd$map, $author$project$Model$RecipeMessage, $author$project$Recipes$Service$fetchAllMetaIngredients),
								A2($elm$core$Platform$Cmd$map, $author$project$Model$RecipeMessage, $author$project$Recipes$Service$fetchUnits),
								A2($elm$core$Platform$Cmd$map, $author$project$Model$RecipeMessage, $author$project$Recipes$Service$fetchRecipes)
							])));
			case 'AddRecipe':
				var save = $author$project$Recipes$Update$mapTab(
					function (r) {
						return $author$project$Model$Recipes(
							_Utils_update(
								r,
								{
									modal: $author$project$Recipes$Model$Add($author$project$Recipes$Model$emptyRecipeEditor)
								}));
					});
				return _Utils_Tuple2(
					A2($author$project$Recipes$Update$updateModel, save, model),
					A2($elm$core$Platform$Cmd$map, $author$project$Model$RecipeMessage, $author$project$Recipes$Service$fetchRecipes));
			case 'EditFilter':
				var filter = msg.a;
				var save = $author$project$Recipes$Update$mapTab(
					function (r) {
						return $author$project$Model$Recipes(
							_Utils_update(
								r,
								{filter: filter}));
					});
				return _Utils_Tuple2(
					A2($author$project$Recipes$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'EditRecipe':
				var id = msg.a;
				var editor = A2(
					$elm$core$Maybe$withDefault,
					$author$project$Recipes$Model$emptyRecipeEditor,
					A2(
						$elm$core$Maybe$map,
						$author$project$Recipes$Model$editorFromReipe,
						A2(
							$elm$core$Maybe$andThen,
							$elm$core$List$head,
							A2(
								$elm$core$Maybe$map,
								$elm$core$List$filter(
									function (r) {
										return _Utils_eq(r.id, id);
									}),
								$author$project$Recipes$Update$recipeList(model)))));
				var save = $author$project$Recipes$Update$mapTab(
					function (r) {
						return $author$project$Model$Recipes(
							_Utils_update(
								r,
								{
									modal: $author$project$Recipes$Model$Edit(editor)
								}));
					});
				return _Utils_Tuple2(
					A2($author$project$Recipes$Update$updateModel, save, model),
					A2(
						$elm$core$Platform$Cmd$map,
						$author$project$Model$RecipeMessage,
						$author$project$Recipes$Service$fetchRecipeIngredients(id)));
			case 'CloseModal':
				var save = $author$project$Recipes$Update$mapTab(
					function (r) {
						return $author$project$Model$Recipes(
							_Utils_update(
								r,
								{modal: $author$project$Recipes$Model$NoModal}));
					});
				return _Utils_Tuple2(
					A2($author$project$Recipes$Update$updateModel, save, model),
					$elm$core$Platform$Cmd$none);
			case 'RecipeChanged':
				var modal = function () {
					var _v1 = model.tabs.active;
					if (_v1.$ === 'Recipes') {
						var r = _v1.a;
						return r.modal;
					} else {
						return $author$project$Recipes$Model$NoModal;
					}
				}();
				return _Utils_Tuple2(
					model,
					A2(
						$elm$core$Platform$Cmd$map,
						$author$project$Model$RecipeMessage,
						$author$project$Recipes$Service$addOrUpdateRecipe(modal)));
			case 'ModalMsg':
				var m = msg.a;
				return A2($author$project$Recipes$Update$handleModalMsg, m, model);
			default:
				return _Utils_Tuple2(model, $elm$core$Platform$Cmd$none);
		}
	});
var $author$project$Recipes$Main$handleRecipesMsg = $author$project$Recipes$Update$handleMsg;
var $author$project$Events$InitTab = {$: 'InitTab'};
var $author$project$Events$init = $author$project$Events$InitTab;
var $author$project$Utils$Cursor$list = function (cursor) {
	return _Utils_ap(
		cursor.left,
		A2($elm$core$List$cons, cursor.active, cursor.right));
};
var $elm$core$Tuple$second = function (_v0) {
	var y = _v0.b;
	return y;
};
var $elm$core$List$drop = F2(
	function (n, list) {
		drop:
		while (true) {
			if (n <= 0) {
				return list;
			} else {
				if (!list.b) {
					return list;
				} else {
					var x = list.a;
					var xs = list.b;
					var $temp$n = n - 1,
						$temp$list = xs;
					n = $temp$n;
					list = $temp$list;
					continue drop;
				}
			}
		}
	});
var $elm$core$List$takeReverse = F3(
	function (n, list, kept) {
		takeReverse:
		while (true) {
			if (n <= 0) {
				return kept;
			} else {
				if (!list.b) {
					return kept;
				} else {
					var x = list.a;
					var xs = list.b;
					var $temp$n = n - 1,
						$temp$list = xs,
						$temp$kept = A2($elm$core$List$cons, x, kept);
					n = $temp$n;
					list = $temp$list;
					kept = $temp$kept;
					continue takeReverse;
				}
			}
		}
	});
var $elm$core$List$takeTailRec = F2(
	function (n, list) {
		return $elm$core$List$reverse(
			A3($elm$core$List$takeReverse, n, list, _List_Nil));
	});
var $elm$core$List$takeFast = F3(
	function (ctr, n, list) {
		if (n <= 0) {
			return _List_Nil;
		} else {
			var _v0 = _Utils_Tuple2(n, list);
			_v0$1:
			while (true) {
				_v0$5:
				while (true) {
					if (!_v0.b.b) {
						return list;
					} else {
						if (_v0.b.b.b) {
							switch (_v0.a) {
								case 1:
									break _v0$1;
								case 2:
									var _v2 = _v0.b;
									var x = _v2.a;
									var _v3 = _v2.b;
									var y = _v3.a;
									return _List_fromArray(
										[x, y]);
								case 3:
									if (_v0.b.b.b.b) {
										var _v4 = _v0.b;
										var x = _v4.a;
										var _v5 = _v4.b;
										var y = _v5.a;
										var _v6 = _v5.b;
										var z = _v6.a;
										return _List_fromArray(
											[x, y, z]);
									} else {
										break _v0$5;
									}
								default:
									if (_v0.b.b.b.b && _v0.b.b.b.b.b) {
										var _v7 = _v0.b;
										var x = _v7.a;
										var _v8 = _v7.b;
										var y = _v8.a;
										var _v9 = _v8.b;
										var z = _v9.a;
										var _v10 = _v9.b;
										var w = _v10.a;
										var tl = _v10.b;
										return (ctr > 1000) ? A2(
											$elm$core$List$cons,
											x,
											A2(
												$elm$core$List$cons,
												y,
												A2(
													$elm$core$List$cons,
													z,
													A2(
														$elm$core$List$cons,
														w,
														A2($elm$core$List$takeTailRec, n - 4, tl))))) : A2(
											$elm$core$List$cons,
											x,
											A2(
												$elm$core$List$cons,
												y,
												A2(
													$elm$core$List$cons,
													z,
													A2(
														$elm$core$List$cons,
														w,
														A3($elm$core$List$takeFast, ctr + 1, n - 4, tl)))));
									} else {
										break _v0$5;
									}
							}
						} else {
							if (_v0.a === 1) {
								break _v0$1;
							} else {
								break _v0$5;
							}
						}
					}
				}
				return list;
			}
			var _v1 = _v0.b;
			var x = _v1.a;
			return _List_fromArray(
				[x]);
		}
	});
var $elm$core$List$take = F2(
	function (n, list) {
		return A3($elm$core$List$takeFast, 0, n, list);
	});
var $author$project$Utils$Cursor$setActive = F2(
	function (index, cursor) {
		var r = A2(
			$elm$core$List$drop,
			index + 1,
			$author$project$Utils$Cursor$list(cursor));
		var newActive = $elm$core$List$head(
			A2(
				$elm$core$List$drop,
				index,
				$author$project$Utils$Cursor$list(cursor)));
		var l = A2(
			$elm$core$List$take,
			index,
			$author$project$Utils$Cursor$list(cursor));
		if (newActive.$ === 'Just') {
			var a = newActive.a;
			return _Utils_update(
				cursor,
				{active: a, left: l, right: r});
		} else {
			return cursor;
		}
	});
var $author$project$Utils$Cursor$setActiveBy = F2(
	function (f, cursor) {
		var indexed = A2(
			$elm$core$List$indexedMap,
			F2(
				function (i, a) {
					return _Utils_Tuple2(i, a);
				}),
			$author$project$Utils$Cursor$list(cursor));
		var _v0 = A2(
			$elm$core$List$filter,
			A2($elm$core$Basics$composeL, f, $elm$core$Tuple$second),
			indexed);
		if (!_v0.b) {
			return cursor;
		} else {
			var _v1 = _v0.a;
			var i = _v1.a;
			return A2($author$project$Utils$Cursor$setActive, i, cursor);
		}
	});
var $author$project$Main$tabName = function (tab) {
	switch (tab.$) {
		case 'Ingredients':
			return 'Ingredients';
		case 'Recipes':
			return 'Recipes';
		default:
			return 'Events';
	}
};
var $author$project$Main$changeTab = F2(
	function (tab, model) {
		var c = A2(
			$author$project$Utils$Cursor$setActiveBy,
			function (t) {
				return _Utils_eq(
					$author$project$Main$tabName(t),
					$author$project$Main$tabName(tab));
			},
			model.tabs);
		return $author$project$Main$initTab(
			_Utils_update(
				model,
				{tabs: c}));
	});
var $author$project$Main$initTab = function (model) {
	var _v2 = model.tabs.active;
	switch (_v2.$) {
		case 'Ingredients':
			return A2(
				$author$project$Main$update,
				$author$project$Model$IngredientMessage($author$project$Ingredients$Model$InitTab),
				model);
		case 'Recipes':
			return A2(
				$author$project$Main$update,
				$author$project$Model$RecipeMessage($author$project$Recipes$Model$InitTab),
				model);
		default:
			return A2(
				$author$project$Main$update,
				$author$project$Model$EventsMessage($author$project$Events$init),
				model);
	}
};
var $author$project$Main$update = F2(
	function (msg, model) {
		switch (msg.$) {
			case 'None':
				return _Utils_Tuple2(model, $elm$core$Platform$Cmd$none);
			case 'ChangeTab':
				var tab = msg.a;
				return A2($author$project$Main$changeTab, tab, model);
			case 'IngredientMessage':
				var m = msg.a;
				return A2($author$project$Ingredients$Main$handleIngredientsMsg, m, model);
			case 'RecipeMessage':
				var m = msg.a;
				return A2($author$project$Recipes$Main$handleRecipesMsg, m, model);
			default:
				var e = msg.a;
				var _v1 = A2($author$project$Events$handleEventTabMsg, e, model.events);
				var events = _v1.a;
				var cmd = _v1.b;
				return _Utils_Tuple2(
					_Utils_update(
						model,
						{events: events}),
					A2($elm$core$Platform$Cmd$map, $author$project$Model$EventsMessage, cmd));
		}
	});
var $elm$html$Html$Attributes$class = $elm$html$Html$Attributes$stringProperty('className');
var $elm$html$Html$div = _VirtualDom_node('div');
var $author$project$Utils$Cursor$active = function (cursor) {
	return cursor.active;
};
var $elm$html$Html$li = _VirtualDom_node('li');
var $elm$virtual_dom$VirtualDom$attribute = F2(
	function (key, value) {
		return A2(
			_VirtualDom_attribute,
			_VirtualDom_noOnOrFormAction(key),
			_VirtualDom_noJavaScriptOrHtmlUri(value));
	});
var $elm$html$Html$Attributes$attribute = $elm$virtual_dom$VirtualDom$attribute;
var $author$project$Utils$Main$role = $elm$html$Html$Attributes$attribute('role');
var $author$project$Navbar$generateNavbarItem = F3(
	function (active, view, tab) {
		return active ? A2(
			$elm$html$Html$li,
			_List_fromArray(
				[
					$elm$html$Html$Events$onClick(
					$author$project$Model$ChangeTab(tab))
				]),
			_List_fromArray(
				[
					A2(
					$elm$html$Html$a,
					_List_fromArray(
						[
							$author$project$Utils$Main$role('button')
						]),
					_List_fromArray(
						[
							$elm$html$Html$text(
							view(tab))
						]))
				])) : A2(
			$elm$html$Html$li,
			_List_fromArray(
				[
					$elm$html$Html$Events$onClick(
					$author$project$Model$ChangeTab(tab))
				]),
			_List_fromArray(
				[
					A2(
					$elm$html$Html$a,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text(
							view(tab))
						]))
				]));
	});
var $author$project$Utils$Cursor$left = function (cursor) {
	return cursor.left;
};
var $elm$html$Html$nav = _VirtualDom_node('nav');
var $author$project$Utils$Cursor$right = function (cursor) {
	return cursor.right;
};
var $elm$html$Html$strong = _VirtualDom_node('strong');
var $elm$html$Html$ul = _VirtualDom_node('ul');
var $author$project$Navbar$generateNavbar = F2(
	function (view, tabs) {
		var r = A2(
			$elm$core$List$map,
			A2($author$project$Navbar$generateNavbarItem, false, view),
			$author$project$Utils$Cursor$right(tabs));
		var l = A2(
			$elm$core$List$map,
			A2($author$project$Navbar$generateNavbarItem, false, view),
			$author$project$Utils$Cursor$left(tabs));
		var a = A3(
			$author$project$Navbar$generateNavbarItem,
			true,
			view,
			$author$project$Utils$Cursor$active(tabs));
		return A2(
			$elm$html$Html$nav,
			_List_Nil,
			_List_fromArray(
				[
					A2(
					$elm$html$Html$ul,
					_List_Nil,
					_List_fromArray(
						[
							A2(
							$elm$html$Html$li,
							_List_Nil,
							_List_fromArray(
								[
									A2(
									$elm$html$Html$strong,
									_List_Nil,
									_List_fromArray(
										[
											$elm$html$Html$text('foodcalc')
										]))
								]))
						])),
					A2(
					$elm$html$Html$ul,
					_List_Nil,
					_Utils_ap(
						l,
						A2($elm$core$List$cons, a, r)))
				]));
	});
var $elm$html$Html$map = $elm$virtual_dom$VirtualDom$map;
var $author$project$SearchList$SetSearch = function (a) {
	return {$: 'SetSearch', a: a};
};
var $author$project$SearchList$filterList = function (searchList) {
	var list = searchList.a.list;
	var search = searchList.a.search;
	var viewFilter = searchList.a.viewFilter;
	return A2(
		$elm$core$List$filter,
		viewFilter(search),
		list);
};
var $elm$html$Html$input = _VirtualDom_node('input');
var $elm$html$Html$Events$alwaysStop = function (x) {
	return _Utils_Tuple2(x, true);
};
var $elm$virtual_dom$VirtualDom$MayStopPropagation = function (a) {
	return {$: 'MayStopPropagation', a: a};
};
var $elm$html$Html$Events$stopPropagationOn = F2(
	function (event, decoder) {
		return A2(
			$elm$virtual_dom$VirtualDom$on,
			event,
			$elm$virtual_dom$VirtualDom$MayStopPropagation(decoder));
	});
var $elm$json$Json$Decode$at = F2(
	function (fields, decoder) {
		return A3($elm$core$List$foldr, $elm$json$Json$Decode$field, decoder, fields);
	});
var $elm$html$Html$Events$targetValue = A2(
	$elm$json$Json$Decode$at,
	_List_fromArray(
		['target', 'value']),
	$elm$json$Json$Decode$string);
var $elm$html$Html$Events$onInput = function (tagger) {
	return A2(
		$elm$html$Html$Events$stopPropagationOn,
		'input',
		A2(
			$elm$json$Json$Decode$map,
			$elm$html$Html$Events$alwaysStop,
			A2($elm$json$Json$Decode$map, tagger, $elm$html$Html$Events$targetValue)));
};
var $author$project$SearchList$viewItem = F2(
	function (searchList, item) {
		var viewContent = searchList.a.viewContent;
		return A2(
			$elm$html$Html$li,
			_List_Nil,
			viewContent(item));
	});
var $author$project$SearchList$view = function (searchList) {
	var mapMsg = searchList.a.mapMsg;
	return A2(
		$elm$html$Html$div,
		_List_Nil,
		_List_fromArray(
			[
				A2(
				$elm$html$Html$input,
				_List_fromArray(
					[
						$elm$html$Html$Events$onInput(
						function (i) {
							return mapMsg(
								$author$project$SearchList$SetSearch(i));
						})
					]),
				_List_Nil),
				A2(
				$elm$html$Html$ul,
				_List_Nil,
				A2(
					$elm$core$List$map,
					$author$project$SearchList$viewItem(searchList),
					$author$project$SearchList$filterList(searchList)))
			]));
};
var $author$project$Events$Budget = function (a) {
	return {$: 'Budget', a: a};
};
var $author$project$Events$CloseModal = {$: 'CloseModal'};
var $author$project$Events$Comment = function (a) {
	return {$: 'Comment', a: a};
};
var $author$project$Events$Name = function (a) {
	return {$: 'Name', a: a};
};
var $author$project$Events$NewMeal = {$: 'NewMeal'};
var $author$project$Events$SaveModal = function (a) {
	return {$: 'SaveModal', a: a};
};
var $author$project$Events$eventBudget = function (event) {
	if (event.$ === 'Exists') {
		var budget = event.a.budget;
		return budget;
	} else {
		var budget = event.a.budget;
		return budget;
	}
};
var $author$project$Events$eventComment = function (event) {
	if (event.$ === 'Exists') {
		var comment = event.a.comment;
		return A2($elm$core$Maybe$withDefault, '', comment);
	} else {
		var comment = event.a.comment;
		return A2($elm$core$Maybe$withDefault, '', comment);
	}
};
var $elm$html$Html$Attributes$placeholder = $elm$html$Html$Attributes$stringProperty('placeholder');
var $elm$html$Html$Attributes$type_ = $elm$html$Html$Attributes$stringProperty('type');
var $elm$html$Html$Attributes$value = $elm$html$Html$Attributes$stringProperty('value');
var $elm$html$Html$article = _VirtualDom_node('article');
var $elm$html$Html$footer = _VirtualDom_node('footer');
var $elm$html$Html$header = _VirtualDom_node('header');
var $elm$virtual_dom$VirtualDom$node = function (tag) {
	return _VirtualDom_node(
		_VirtualDom_noScript(tag));
};
var $elm$html$Html$node = $elm$virtual_dom$VirtualDom$node;
var $elm$html$Html$p = _VirtualDom_node('p');
var $feathericons$elm_feather$FeatherIcons$x = A2(
	$feathericons$elm_feather$FeatherIcons$makeBuilder,
	'x',
	_List_fromArray(
		[
			A2(
			$elm$svg$Svg$line,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$x1('18'),
					$elm$svg$Svg$Attributes$y1('6'),
					$elm$svg$Svg$Attributes$x2('6'),
					$elm$svg$Svg$Attributes$y2('18')
				]),
			_List_Nil),
			A2(
			$elm$svg$Svg$line,
			_List_fromArray(
				[
					$elm$svg$Svg$Attributes$x1('6'),
					$elm$svg$Svg$Attributes$y1('6'),
					$elm$svg$Svg$Attributes$x2('18'),
					$elm$svg$Svg$Attributes$y2('18')
				]),
			_List_Nil)
		]));
var $author$project$Modal$viewModal = F4(
	function (title, onClose, footer, content) {
		return A3(
			$elm$html$Html$node,
			'dialog',
			_List_fromArray(
				[
					A2($elm$html$Html$Attributes$attribute, 'open', '')
				]),
			_List_fromArray(
				[
					A2(
					$elm$html$Html$article,
					_List_Nil,
					_List_fromArray(
						[
							A2(
							$elm$html$Html$header,
							_List_Nil,
							_List_fromArray(
								[
									A2(
									$elm$html$Html$a,
									_List_fromArray(
										[
											$elm$html$Html$Events$onClick(onClose),
											$elm$html$Html$Attributes$href('#')
										]),
									_List_fromArray(
										[
											A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$x)
										])),
									$elm$html$Html$text(title)
								])),
							A2(
							$elm$html$Html$p,
							_List_fromArray(
								[
									$elm$html$Html$Attributes$class('container')
								]),
							content),
							A2($elm$html$Html$footer, _List_Nil, footer)
						]))
				]));
	});
var $author$project$Events$viewEventDetails = function (evDetails) {
	var event = evDetails.a.event;
	var details = evDetails.a.details;
	var meals = function () {
		switch (details.$) {
			case 'NotAsked':
				return A2(
					$elm$html$Html$div,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('NotAsked')
						]));
			case 'Loading':
				return A2(
					$elm$html$Html$div,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('Loading')
						]));
			case 'Failure':
				return A2(
					$elm$html$Html$div,
					_List_Nil,
					_List_fromArray(
						[
							$elm$html$Html$text('Error loading Meals')
						]));
			default:
				var searchList = details.a;
				return $author$project$SearchList$view(
					A2(
						$author$project$SearchList$addAll,
						_List_fromArray(
							[$author$project$Events$NewMeal]),
						searchList));
		}
	}();
	var buttons = _List_fromArray(
		[
			A2(
			$elm$html$Html$a,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$href('#'),
					$elm$html$Html$Events$onClick($author$project$Events$CloseModal),
					$author$project$Utils$Main$role('button'),
					$elm$html$Html$Attributes$class('secondary')
				]),
			_List_fromArray(
				[
					$elm$html$Html$text('Close')
				])),
			A2(
			$elm$html$Html$a,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$href('#'),
					$elm$html$Html$Events$onClick(
					$author$project$Events$SaveModal(evDetails)),
					$author$project$Utils$Main$role('button')
				]),
			_List_fromArray(
				[
					$elm$html$Html$text('Save')
				]))
		]);
	var _v1 = _Utils_Tuple3(
		A2(
			$elm$html$Html$input,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$type_('text'),
					$elm$html$Html$Attributes$placeholder('Name'),
					$elm$html$Html$Events$onInput(
					function (name) {
						return A2(
							$author$project$Events$EventDetails,
							event,
							$author$project$Events$Name(name));
					}),
					$elm$html$Html$Attributes$value(
					$author$project$Events$eventName(event))
				]),
			_List_Nil),
		A2(
			$elm$html$Html$input,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$type_('text'),
					$elm$html$Html$Attributes$placeholder('Budget'),
					$elm$html$Html$Events$onInput(
					function (budget) {
						return A2(
							$author$project$Events$EventDetails,
							event,
							$author$project$Events$Budget(budget));
					}),
					$elm$html$Html$Attributes$value(
					$author$project$Events$eventBudget(event))
				]),
			_List_Nil),
		A2(
			$elm$html$Html$input,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$type_('text'),
					$elm$html$Html$Attributes$placeholder('Comment'),
					$elm$html$Html$Events$onInput(
					function (comment) {
						return A2(
							$author$project$Events$EventDetails,
							event,
							$author$project$Events$Comment(comment));
					}),
					$elm$html$Html$Attributes$value(
					$author$project$Events$eventComment(event))
				]),
			_List_Nil));
	var nameField = _v1.a;
	var budgetField = _v1.b;
	var commentField = _v1.c;
	var fields = _List_fromArray(
		[
			A2(
			$elm$html$Html$div,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$class('grid')
				]),
			_List_fromArray(
				[nameField, budgetField])),
			commentField
		]);
	return A4(
		$author$project$Modal$viewModal,
		'Event Details',
		$author$project$Events$CloseModal,
		buttons,
		_Utils_ap(
			fields,
			_List_fromArray(
				[meals])));
};
var $author$project$Events$viewEvents = function (data) {
	var events = data.a.events;
	var eventModal = data.a.eventModal;
	switch (events.$) {
		case 'NotAsked':
			return A2(
				$elm$html$Html$div,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text('NotAsked')
					]));
		case 'Loading':
			return A2(
				$elm$html$Html$div,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text('Loading')
					]));
		case 'Failure':
			return A2(
				$elm$html$Html$div,
				_List_Nil,
				_List_fromArray(
					[
						$elm$html$Html$text('Error loading Events')
					]));
		default:
			var searchList = events.a;
			if (eventModal.$ === 'Just') {
				var m = eventModal.a;
				return A2(
					$elm$html$Html$div,
					_List_Nil,
					_List_fromArray(
						[
							$author$project$SearchList$view(searchList),
							$author$project$Events$viewEventDetails(m)
						]));
			} else {
				return A2(
					$elm$html$Html$div,
					_List_Nil,
					_List_fromArray(
						[
							$author$project$SearchList$view(searchList)
						]));
			}
	}
};
var $author$project$Ingredients$Model$AddIngredient = {$: 'AddIngredient'};
var $author$project$Ingredients$Model$EditFilter = function (a) {
	return {$: 'EditFilter', a: a};
};
var $elm$html$Html$table = _VirtualDom_node('table');
var $elm$html$Html$tbody = _VirtualDom_node('tbody');
var $elm$html$Html$td = _VirtualDom_node('td');
var $elm$html$Html$tr = _VirtualDom_node('tr');
var $author$project$Utils$View$listView = F2(
	function (row, list) {
		var rows = A2(
			$elm$core$List$map,
			A2(
				$elm$core$Basics$composeL,
				A2(
					$elm$core$Basics$composeL,
					$elm$html$Html$tr(_List_Nil),
					$elm$core$List$map(
						function (x) {
							return A2(
								$elm$html$Html$td,
								_List_Nil,
								_List_fromArray(
									[x]));
						})),
				row),
			list);
		return A2(
			$elm$html$Html$table,
			_List_fromArray(
				[
					$author$project$Utils$Main$role('grid')
				]),
			_List_fromArray(
				[
					A2($elm$html$Html$tbody, _List_Nil, rows)
				]));
	});
var $elm$html$Html$button = _VirtualDom_node('button');
var $author$project$Utils$View$searchBar = F2(
	function (filterChange, add) {
		return A2(
			$elm$html$Html$table,
			_List_Nil,
			_List_fromArray(
				[
					A2(
					$elm$html$Html$tr,
					_List_Nil,
					_List_fromArray(
						[
							A2(
							$elm$html$Html$td,
							_List_Nil,
							_List_fromArray(
								[
									A2(
									$elm$html$Html$input,
									_List_fromArray(
										[
											$elm$html$Html$Attributes$class('search'),
											$elm$html$Html$Attributes$type_('text'),
											$elm$html$Html$Attributes$placeholder('Search'),
											$elm$html$Html$Events$onInput(filterChange)
										]),
									_List_Nil)
								])),
							A2(
							$elm$html$Html$td,
							_List_Nil,
							_List_fromArray(
								[
									A2(
									$elm$html$Html$button,
									_List_fromArray(
										[
											$elm$html$Html$Events$onClick(add)
										]),
									_List_fromArray(
										[
											A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$plus)
										]))
								]))
						]))
				]));
	});
var $author$project$Utils$View$filterListView = F2(
	function (options, list) {
		var filtered = A2($elm$core$List$filter, options.filter, list);
		return A2(
			$elm$html$Html$div,
			_List_Nil,
			_List_fromArray(
				[
					A2($author$project$Utils$View$searchBar, options.filterChange, options.onAdd),
					A2($author$project$Utils$View$listView, options.row, filtered)
				]));
	});
var $author$project$Ingredients$Model$EditComment = function (a) {
	return {$: 'EditComment', a: a};
};
var $author$project$Ingredients$Model$EditEnergy = function (a) {
	return {$: 'EditEnergy', a: a};
};
var $author$project$Ingredients$Model$EditName = function (a) {
	return {$: 'EditName', a: a};
};
var $author$project$Ingredients$Model$ModalMsg = function (a) {
	return {$: 'ModalMsg', a: a};
};
var $author$project$Ingredients$Model$Save = function (a) {
	return {$: 'Save', a: a};
};
var $elm$html$Html$h3 = _VirtualDom_node('h3');
var $author$project$Ingredients$View$ingredientDetails = F3(
	function (submit, title, ingredient) {
		var id_text = function () {
			var _v0 = ingredient.id;
			if (_v0.$ === 'Nothing') {
				return '';
			} else {
				var i = _v0.a;
				return ' (id: ' + ($elm$core$String$fromInt(i) + ')');
			}
		}();
		return A3(
			$elm$html$Html$node,
			'dialog',
			_List_fromArray(
				[
					A2($elm$html$Html$Attributes$attribute, 'open', '')
				]),
			_List_fromArray(
				[
					A2(
					$elm$html$Html$article,
					_List_Nil,
					_List_fromArray(
						[
							A2(
							$elm$html$Html$a,
							_List_fromArray(
								[
									$elm$html$Html$Events$onClick(
									$author$project$Model$IngredientMessage($author$project$Ingredients$Model$CloseModal))
								]),
							_List_fromArray(
								[
									A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$x)
								])),
							A2(
							$elm$html$Html$h3,
							_List_Nil,
							_List_fromArray(
								[
									$elm$html$Html$text(
									_Utils_ap(title, id_text))
								])),
							A2(
							$elm$html$Html$div,
							_List_fromArray(
								[
									$elm$html$Html$Attributes$class('grid')
								]),
							_List_fromArray(
								[
									A2(
									$elm$html$Html$input,
									_List_fromArray(
										[
											$elm$html$Html$Attributes$type_('text'),
											$elm$html$Html$Attributes$placeholder('name'),
											$elm$html$Html$Events$onInput(
											A2(
												$elm$core$Basics$composeL,
												A2($elm$core$Basics$composeL, $author$project$Model$IngredientMessage, $author$project$Ingredients$Model$ModalMsg),
												$author$project$Ingredients$Model$EditName)),
											$elm$html$Html$Attributes$value(ingredient.name)
										]),
									_List_Nil),
									A2(
									$elm$html$Html$input,
									_List_fromArray(
										[
											$elm$html$Html$Attributes$type_('number'),
											$elm$html$Html$Attributes$placeholder('energy'),
											$elm$html$Html$Events$onInput(
											A2(
												$elm$core$Basics$composeL,
												A2($elm$core$Basics$composeL, $author$project$Model$IngredientMessage, $author$project$Ingredients$Model$ModalMsg),
												$author$project$Ingredients$Model$EditEnergy)),
											$elm$html$Html$Attributes$value(ingredient.energy)
										]),
									_List_Nil)
								])),
							A2(
							$elm$html$Html$input,
							_List_fromArray(
								[
									$elm$html$Html$Attributes$type_('text'),
									$elm$html$Html$Attributes$placeholder('comment'),
									$elm$html$Html$Events$onInput(
									A2(
										$elm$core$Basics$composeL,
										A2($elm$core$Basics$composeL, $author$project$Model$IngredientMessage, $author$project$Ingredients$Model$ModalMsg),
										$author$project$Ingredients$Model$EditComment)),
									$elm$html$Html$Attributes$value(ingredient.comment)
								]),
							_List_Nil),
							A2(
							$elm$html$Html$footer,
							_List_fromArray(
								[
									$elm$html$Html$Attributes$class('grid')
								]),
							_List_fromArray(
								[
									A2(
									$elm$html$Html$button,
									_List_fromArray(
										[
											$elm$html$Html$Events$onClick(
											$author$project$Model$IngredientMessage($author$project$Ingredients$Model$CloseModal))
										]),
									_List_fromArray(
										[
											$elm$html$Html$text('Cancel')
										])),
									A2(
									$elm$html$Html$button,
									_List_fromArray(
										[
											$elm$html$Html$Events$onClick(
											$author$project$Model$IngredientMessage(
												$author$project$Ingredients$Model$ModalMsg(
													$author$project$Ingredients$Model$Save(ingredient))))
										]),
									_List_fromArray(
										[
											$elm$html$Html$text(submit)
										]))
								]))
						]))
				]));
	});
var $author$project$Ingredients$View$modal = function (m) {
	switch (m.$) {
		case 'Add':
			var ingredient = m.a;
			return A3($author$project$Ingredients$View$ingredientDetails, 'Add', 'Add ingredient', ingredient);
		case 'Edit':
			var ingredient = m.a;
			return A3($author$project$Ingredients$View$ingredientDetails, 'Save', 'Edit ingredient', ingredient);
		default:
			return A3($elm$html$Html$node, 'dialog', _List_Nil, _List_Nil);
	}
};
var $author$project$Ingredients$Model$DeleteIngredient = function (a) {
	return {$: 'DeleteIngredient', a: a};
};
var $author$project$Ingredients$Model$EditIngredient = function (a) {
	return {$: 'EditIngredient', a: a};
};
var $author$project$Ingredients$View$renderSingleIngredient = function (ingredient) {
	return _List_fromArray(
		[
			$elm$html$Html$text(
			$elm$core$String$fromInt(ingredient.id)),
			$elm$html$Html$text(ingredient.name),
			$elm$html$Html$text(
			$elm$core$String$fromFloat(ingredient.energy)),
			$elm$html$Html$text(
			A2($elm$core$Maybe$withDefault, '', ingredient.comment)),
			A2(
			$elm$html$Html$a,
			_List_fromArray(
				[
					$elm$html$Html$Events$onClick(
					$author$project$Model$IngredientMessage(
						$author$project$Ingredients$Model$EditIngredient(ingredient.id)))
				]),
			_List_fromArray(
				[
					A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$edit)
				])),
			A2(
			$elm$html$Html$a,
			_List_fromArray(
				[
					$elm$html$Html$Events$onClick(
					$author$project$Model$IngredientMessage(
						$author$project$Ingredients$Model$DeleteIngredient(ingredient.id)))
				]),
			_List_fromArray(
				[
					A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$trash2)
				]))
		]);
};
var $author$project$Ingredients$View$view = function (ingredients) {
	var list = function () {
		var _v0 = ingredients.ingredients;
		switch (_v0.$) {
			case 'NotAsked':
				return $elm$html$Html$text('Not Asked');
			case 'Loading':
				return $elm$html$Html$text('Loading');
			case 'Success':
				var is = _v0.a;
				return A2(
					$author$project$Utils$View$filterListView,
					{
						filter: function (i) {
							return A2(
								$elm$core$String$contains,
								$elm$core$String$toLower(ingredients.filter),
								$elm$core$String$toLower(i.name));
						},
						filterChange: A2($elm$core$Basics$composeL, $author$project$Model$IngredientMessage, $author$project$Ingredients$Model$EditFilter),
						onAdd: $author$project$Model$IngredientMessage($author$project$Ingredients$Model$AddIngredient),
						row: $author$project$Ingredients$View$renderSingleIngredient
					},
					is);
			default:
				return $elm$html$Html$text('Failure');
		}
	}();
	return A2(
		$elm$html$Html$div,
		_List_Nil,
		_List_fromArray(
			[
				list,
				$author$project$Ingredients$View$modal(ingredients.modal)
			]));
};
var $author$project$Ingredients$Main$viewIngredients = $author$project$Ingredients$View$view;
var $author$project$Recipes$Model$AddRecipe = {$: 'AddRecipe'};
var $author$project$Recipes$Model$EditFilter = function (a) {
	return {$: 'EditFilter', a: a};
};
var $author$project$Recipes$Model$CloseModal = {$: 'CloseModal'};
var $author$project$Recipes$Model$EditComment = function (a) {
	return {$: 'EditComment', a: a};
};
var $author$project$Recipes$Model$EditName = function (a) {
	return {$: 'EditName', a: a};
};
var $author$project$Recipes$Model$ModalMsg = function (a) {
	return {$: 'ModalMsg', a: a};
};
var $author$project$Recipes$Model$RecipeChanged = function (a) {
	return {$: 'RecipeChanged', a: a};
};
var $author$project$Recipes$Model$AddMetaIngredient = function (a) {
	return {$: 'AddMetaIngredient', a: a};
};
var $author$project$Recipes$Model$Delete = {$: 'Delete'};
var $author$project$Recipes$Model$EditMetaIngredient = F2(
	function (a, b) {
		return {$: 'EditMetaIngredient', a: a, b: b};
	});
var $author$project$Recipes$Model$SetAmount = function (a) {
	return {$: 'SetAmount', a: a};
};
var $author$project$Recipes$Model$SetIngredient = function (a) {
	return {$: 'SetIngredient', a: a};
};
var $author$project$Recipes$Model$SetIngredientFilter = function (a) {
	return {$: 'SetIngredientFilter', a: a};
};
var $author$project$Recipes$Model$SetUnit = function (a) {
	return {$: 'SetUnit', a: a};
};
var $author$project$Recipes$Model$SetUnitFilter = function (a) {
	return {$: 'SetUnitFilter', a: a};
};
var $author$project$Recipes$Model$NewId = {$: 'NewId'};
var $author$project$Recipes$Model$SubRecipeId = function (a) {
	return {$: 'SubRecipeId', a: a};
};
var $author$project$Recipes$Model$getId = function (ing) {
	if (ing.$ === 'Just') {
		var ig = ing.a;
		var _v1 = ig.metaIngredient;
		if (_v1.$ === 'IsDirect') {
			var i = _v1.a;
			return $author$project$Recipes$Model$IngredientId(i.id);
		} else {
			var r = _v1.a;
			return $author$project$Recipes$Model$SubRecipeId(r.id);
		}
	} else {
		return $author$project$Recipes$Model$NewId;
	}
};
var $author$project$Recipes$ViewModal$metaIngredientName = function (ig) {
	return A2(
		$elm$core$Maybe$withDefault,
		'',
		A2(
			$elm$core$Maybe$map,
			function (e) {
				if (e.$ === 'IsDirect') {
					var i = e.a;
					return i.name;
				} else {
					var r = e.a;
					return r.name;
				}
			},
			ig));
};
var $elm$html$Html$details = _VirtualDom_node('details');
var $author$project$Utils$Main$nameFilter = F2(
	function (filter, name) {
		return A2(
			$elm$core$String$contains,
			$elm$core$String$toLower(filter),
			$elm$core$String$toLower(name));
	});
var $author$project$Utils$View$search = F2(
	function (action, value) {
		return A2(
			$elm$html$Html$input,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$class('search'),
					$elm$html$Html$Attributes$type_('text'),
					$elm$html$Html$Attributes$placeholder('Filter...'),
					$elm$html$Html$Events$onInput(action),
					$elm$html$Html$Attributes$value(value)
				]),
			_List_Nil);
	});
var $elm$html$Html$summary = _VirtualDom_node('summary');
var $author$project$Utils$View$searchableDropdown = F3(
	function (data, ev, list) {
		var selectedProperty = A2(
			$elm$core$Maybe$withDefault,
			'',
			A2($elm$core$Maybe$map, ev.property, data.selected));
		var filteredList = A2(
			$elm$core$List$filter,
			A2(
				$elm$core$Basics$composeL,
				$author$project$Utils$Main$nameFilter(data.filter),
				ev.property),
			list);
		var options = A2(
			$elm$core$List$map,
			function (x) {
				return A2(
					$elm$html$Html$li,
					_List_Nil,
					_List_fromArray(
						[
							A2(
							$elm$html$Html$a,
							_List_fromArray(
								[
									$elm$html$Html$Events$onClick(
									ev.onSelect(x))
								]),
							_List_fromArray(
								[
									$elm$html$Html$text(
									ev.property(x))
								]))
						]));
			},
			filteredList);
		return A2(
			$elm$html$Html$details,
			_List_fromArray(
				[
					$author$project$Utils$Main$role('list')
				]),
			_List_fromArray(
				[
					A2(
					$elm$html$Html$summary,
					_List_fromArray(
						[
							A2($elm$html$Html$Attributes$attribute, 'aria-haspopup', 'listbox')
						]),
					_List_fromArray(
						[
							$elm$html$Html$text(selectedProperty)
						])),
					A2(
					$elm$html$Html$ul,
					_List_fromArray(
						[
							$author$project$Utils$Main$role('listbox')
						]),
					A2(
						$elm$core$List$cons,
						A2($author$project$Utils$View$search, ev.onFilter, data.filter),
						options))
				]));
	});
var $author$project$Recipes$ViewModal$webDataList = function (data) {
	if (data.$ === 'Success') {
		var items = data.a;
		return items;
	} else {
		return _List_Nil;
	}
};
var $author$project$Recipes$ViewModal$renderRecipeIngredient = F2(
	function (data, ingredientEditor) {
		var ingredient = A2($elm$core$Maybe$map, $elm$core$Tuple$first, ingredientEditor);
		var msg = function () {
			if (ingredient.$ === 'Just') {
				return A2(
					$elm$core$Basics$composeL,
					A2($elm$core$Basics$composeL, $author$project$Model$RecipeMessage, $author$project$Recipes$Model$ModalMsg),
					$author$project$Recipes$Model$EditMetaIngredient(
						$author$project$Recipes$Model$getId(ingredient)));
			} else {
				return A2(
					$elm$core$Basics$composeL,
					A2($elm$core$Basics$composeL, $author$project$Model$RecipeMessage, $author$project$Recipes$Model$ModalMsg),
					$author$project$Recipes$Model$AddMetaIngredient);
			}
		}();
		var editor = A2($elm$core$Maybe$map, $elm$core$Tuple$second, ingredientEditor);
		var iDropdownData = A2(
			$elm$core$Maybe$withDefault,
			$author$project$Utils$Model$newDropdownData($elm$core$Maybe$Nothing),
			A2(
				$elm$core$Maybe$map,
				function (e) {
					return e.ingredientDropdown;
				},
				editor));
		var ingredientDropdown = A3(
			$author$project$Utils$View$searchableDropdown,
			iDropdownData,
			{
				onFilter: A2($elm$core$Basics$composeL, msg, $author$project$Recipes$Model$SetIngredientFilter),
				onSelect: A2($elm$core$Basics$composeL, msg, $author$project$Recipes$Model$SetIngredient),
				property: A2($elm$core$Basics$composeL, $author$project$Recipes$ViewModal$metaIngredientName, $elm$core$Maybe$Just)
			},
			$author$project$Recipes$ViewModal$webDataList(data.allIngredients));
		var uDropdownData = A2(
			$elm$core$Maybe$withDefault,
			$author$project$Utils$Model$newDropdownData($elm$core$Maybe$Nothing),
			A2(
				$elm$core$Maybe$map,
				function (e) {
					return e.unitDropdown;
				},
				editor));
		var unitDropdown = A3(
			$author$project$Utils$View$searchableDropdown,
			uDropdownData,
			{
				onFilter: A2($elm$core$Basics$composeL, msg, $author$project$Recipes$Model$SetUnitFilter),
				onSelect: A2($elm$core$Basics$composeL, msg, $author$project$Recipes$Model$SetUnit),
				property: function (u) {
					return u.name;
				}
			},
			$author$project$Recipes$ViewModal$webDataList(data.allUnits));
		var editAmount = A2(
			$elm$html$Html$input,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$class('amount'),
					$elm$html$Html$Attributes$type_('text'),
					$elm$html$Html$Attributes$placeholder('Amount'),
					$elm$html$Html$Events$onInput(
					A2($elm$core$Basics$composeL, msg, $author$project$Recipes$Model$SetAmount))
				]),
			_List_Nil);
		var deleteButton = A2(
			$elm$html$Html$button,
			_List_fromArray(
				[
					$elm$html$Html$Attributes$class('delete'),
					$elm$html$Html$Events$onClick(
					msg($author$project$Recipes$Model$Delete))
				]),
			_List_fromArray(
				[
					A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$trash2)
				]));
		if (ingredient.$ === 'Just') {
			return _List_fromArray(
				[ingredientDropdown, editAmount, unitDropdown, deleteButton]);
		} else {
			return _List_fromArray(
				[ingredientDropdown]);
		}
	});
var $author$project$Recipes$ViewModal$recipeIngredientsList = F2(
	function (data, editor) {
		var _v0 = editor.ingredients;
		switch (_v0.$) {
			case 'NotAsked':
				return $elm$html$Html$text('Loading ingredients not initiated');
			case 'Loading':
				return $elm$html$Html$text('Loading ingredients ...');
			case 'Failure':
				return $elm$html$Html$text('Error loading ingredients');
			default:
				var ingredients = _v0.a;
				return A2(
					$author$project$Utils$View$listView,
					$author$project$Recipes$ViewModal$renderRecipeIngredient(data),
					_Utils_ap(
						A2($elm$core$List$map, $elm$core$Maybe$Just, ingredients),
						_List_fromArray(
							[$elm$core$Maybe$Nothing])));
		}
	});
var $author$project$Recipes$ViewModal$recipeDetails = F4(
	function (data, submit, title, editor) {
		var id_text = function () {
			var _v0 = editor.id;
			if (_v0.$ === 'Just') {
				var i = _v0.a;
				return ' (id: ' + ($elm$core$String$fromInt(i) + ')');
			} else {
				return '';
			}
		}();
		return A3(
			$elm$html$Html$node,
			'dialog',
			_List_fromArray(
				[
					A2($elm$html$Html$Attributes$attribute, 'open', '')
				]),
			_List_fromArray(
				[
					A2(
					$elm$html$Html$article,
					_List_Nil,
					_List_fromArray(
						[
							A2(
							$elm$html$Html$header,
							_List_Nil,
							_List_fromArray(
								[
									A2(
									$elm$html$Html$a,
									_List_fromArray(
										[
											$elm$html$Html$Events$onClick(
											$author$project$Model$RecipeMessage($author$project$Recipes$Model$CloseModal)),
											$elm$html$Html$Attributes$href('#')
										]),
									_List_fromArray(
										[
											A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$x)
										])),
									A2(
									$elm$html$Html$h3,
									_List_Nil,
									_List_fromArray(
										[
											$elm$html$Html$text(
											_Utils_ap(title, id_text))
										]))
								])),
							A2(
							$elm$html$Html$p,
							_List_fromArray(
								[
									$elm$html$Html$Attributes$class('container')
								]),
							_List_fromArray(
								[
									A2(
									$elm$html$Html$input,
									_List_fromArray(
										[
											$elm$html$Html$Attributes$class('name'),
											$elm$html$Html$Attributes$type_('text'),
											$elm$html$Html$Attributes$placeholder('Name'),
											$elm$html$Html$Events$onInput(
											A2(
												$elm$core$Basics$composeL,
												A2($elm$core$Basics$composeL, $author$project$Model$RecipeMessage, $author$project$Recipes$Model$ModalMsg),
												$author$project$Recipes$Model$EditName)),
											$elm$html$Html$Attributes$value(editor.name)
										]),
									_List_Nil),
									A2(
									$elm$html$Html$input,
									_List_fromArray(
										[
											$elm$html$Html$Attributes$class('comment'),
											$elm$html$Html$Attributes$type_('text'),
											$elm$html$Html$Attributes$placeholder('Comment'),
											$elm$html$Html$Events$onInput(
											A2(
												$elm$core$Basics$composeL,
												A2($elm$core$Basics$composeL, $author$project$Model$RecipeMessage, $author$project$Recipes$Model$ModalMsg),
												$author$project$Recipes$Model$EditComment)),
											$elm$html$Html$Attributes$value(
											A2($elm$core$Maybe$withDefault, '', editor.comment))
										]),
									_List_Nil),
									A2($author$project$Recipes$ViewModal$recipeIngredientsList, data, editor)
								])),
							A2(
							$elm$html$Html$footer,
							_List_Nil,
							_List_fromArray(
								[
									A2(
									$elm$html$Html$a,
									_List_fromArray(
										[
											$author$project$Utils$Main$role('button'),
											$elm$html$Html$Attributes$class('secondary'),
											$elm$html$Html$Events$onClick(
											$author$project$Model$RecipeMessage($author$project$Recipes$Model$CloseModal)),
											$elm$html$Html$Attributes$href('#')
										]),
									_List_fromArray(
										[
											$elm$html$Html$text('Cancel')
										])),
									A2(
									$elm$html$Html$a,
									_List_fromArray(
										[
											$author$project$Utils$Main$role('button'),
											$elm$html$Html$Events$onClick(
											$author$project$Model$RecipeMessage(
												$author$project$Recipes$Model$RecipeChanged(editor))),
											$elm$html$Html$Attributes$href('#')
										]),
									_List_fromArray(
										[
											$elm$html$Html$text(submit)
										]))
								]))
						]))
				]));
	});
var $author$project$Recipes$ViewModal$modal = function (data) {
	var _v0 = data.modal;
	switch (_v0.$) {
		case 'NoModal':
			return A3($elm$html$Html$node, 'dialog', _List_Nil, _List_Nil);
		case 'Add':
			var recipe = _v0.a;
			return A4($author$project$Recipes$ViewModal$recipeDetails, data, 'Add', 'Add recipe', recipe);
		default:
			var recipe = _v0.a;
			return A4($author$project$Recipes$ViewModal$recipeDetails, data, 'Save', 'Edit recipe', recipe);
	}
};
var $author$project$Recipes$Model$DeleteRecipe = function (a) {
	return {$: 'DeleteRecipe', a: a};
};
var $author$project$Recipes$Model$EditRecipe = function (a) {
	return {$: 'EditRecipe', a: a};
};
var $author$project$Recipes$View$renderRecipe = function (recipe) {
	return _List_fromArray(
		[
			$elm$html$Html$text(
			$elm$core$String$fromInt(recipe.id)),
			$elm$html$Html$text(recipe.name),
			$elm$html$Html$text(
			A2($elm$core$Maybe$withDefault, '', recipe.comment)),
			A2(
			$elm$html$Html$a,
			_List_fromArray(
				[
					$elm$html$Html$Events$onClick(
					$author$project$Model$RecipeMessage(
						$author$project$Recipes$Model$EditRecipe(recipe.id)))
				]),
			_List_fromArray(
				[
					A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$edit)
				])),
			A2(
			$elm$html$Html$a,
			_List_fromArray(
				[
					$elm$html$Html$Events$onClick(
					$author$project$Model$RecipeMessage(
						$author$project$Recipes$Model$DeleteRecipe(recipe.id)))
				]),
			_List_fromArray(
				[
					A2($feathericons$elm_feather$FeatherIcons$toHtml, _List_Nil, $feathericons$elm_feather$FeatherIcons$trash2)
				]))
		]);
};
var $author$project$Recipes$View$view = function (recipeData) {
	var list = function () {
		var _v0 = recipeData.recipes;
		switch (_v0.$) {
			case 'NotAsked':
				return $elm$html$Html$text('Loading not initiated');
			case 'Loading':
				return $elm$html$Html$text('Loading...');
			case 'Failure':
				return $elm$html$Html$text('Error loading recipes');
			default:
				var recipes = _v0.a;
				return A2(
					$author$project$Utils$View$filterListView,
					{
						filter: function (r) {
							return A2($author$project$Utils$Main$nameFilter, recipeData.filter, r.name);
						},
						filterChange: A2($elm$core$Basics$composeL, $author$project$Model$RecipeMessage, $author$project$Recipes$Model$EditFilter),
						onAdd: $author$project$Model$RecipeMessage($author$project$Recipes$Model$AddRecipe),
						row: $author$project$Recipes$View$renderRecipe
					},
					recipes);
		}
	}();
	return A2(
		$elm$html$Html$div,
		_List_Nil,
		_List_fromArray(
			[
				list,
				$author$project$Recipes$ViewModal$modal(recipeData)
			]));
};
var $author$project$Recipes$Main$viewRecipes = $author$project$Recipes$View$view;
var $author$project$Main$renderSelectedView = function (model) {
	var _v0 = $author$project$Utils$Cursor$active(model.tabs);
	switch (_v0.$) {
		case 'Ingredients':
			var i = _v0.a;
			return $author$project$Ingredients$Main$viewIngredients(i);
		case 'Recipes':
			var r = _v0.a;
			return $author$project$Recipes$Main$viewRecipes(r);
		default:
			return A2(
				$elm$html$Html$map,
				$author$project$Model$EventsMessage,
				$author$project$Events$viewEvents(model.events));
	}
};
var $author$project$Main$view = function (model) {
	return A2(
		$elm$html$Html$div,
		_List_fromArray(
			[
				$elm$html$Html$Attributes$class('container')
			]),
		_List_fromArray(
			[
				A2($author$project$Navbar$generateNavbar, $author$project$Main$tabName, model.tabs),
				$author$project$Main$renderSelectedView(model)
			]));
};
var $author$project$Main$main = $elm$browser$Browser$element(
	{init: $author$project$Main$init, subscriptions: $author$project$Main$subscriptions, update: $author$project$Main$update, view: $author$project$Main$view});
_Platform_export({'Main':{'init':$author$project$Main$main(
	$elm$json$Json$Decode$succeed(_Utils_Tuple0))(0)}});}(this));