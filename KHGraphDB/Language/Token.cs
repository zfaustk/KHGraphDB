using System;
using System.Collections.Generic;
using System.Text;

namespace KHGraphDB.Language
{
    public enum TokenKind
    {
        Eof,
        Ident,
        String,
        Number,
        LParen,
        RParen,
        LBrack,
        RBrack,
        LBrace,
        RBrace,
        Colon,
        Comma,
        Dot,
        Eq,
        Dash,
        Arrow,
        LArrow
    }

    public sealed class Token
    {
        public TokenKind Kind;
        public string Text;
        public int Pos;

        public Token(TokenKind kind, string text, int pos)
        {
            Kind = kind;
            Text = text;
            Pos = pos;
        }
    }

    /// <summary>
    /// Cypher-shaped scanner. No regex. Keywords are identifiers.
    /// </summary>
    public sealed class Lexer
    {
        readonly string _s;
        int _i;

        public Lexer(string text)
        {
            _s = text ?? "";
            _i = 0;
        }

        public Token Next()
        {
            SkipSpace();
            if (_i >= _s.Length)
                return new Token(TokenKind.Eof, "", _i);

            int pos = _i;
            char c = _s[_i];

            if (c == '(') { _i++; return new Token(TokenKind.LParen, "(", pos); }
            if (c == ')') { _i++; return new Token(TokenKind.RParen, ")", pos); }
            if (c == '[') { _i++; return new Token(TokenKind.LBrack, "[", pos); }
            if (c == ']') { _i++; return new Token(TokenKind.RBrack, "]", pos); }
            if (c == '{') { _i++; return new Token(TokenKind.LBrace, "{", pos); }
            if (c == '}') { _i++; return new Token(TokenKind.RBrace, "}", pos); }
            if (c == ':') { _i++; return new Token(TokenKind.Colon, ":", pos); }
            if (c == ',') { _i++; return new Token(TokenKind.Comma, ",", pos); }
            if (c == '.') { _i++; return new Token(TokenKind.Dot, ".", pos); }
            if (c == '=') { _i++; return new Token(TokenKind.Eq, "=", pos); }

            if (c == '<')
            {
                if (_i + 1 < _s.Length && _s[_i + 1] == '-')
                {
                    _i += 2;
                    return new Token(TokenKind.LArrow, "<-", pos);
                }
            }

            if (c == '-')
            {
                if (_i + 1 < _s.Length && _s[_i + 1] == '>')
                {
                    _i += 2;
                    return new Token(TokenKind.Arrow, "->", pos);
                }
                _i++;
                return new Token(TokenKind.Dash, "-", pos);
            }

            if (c == '"' || c == '\'')
                return ReadString(c, pos);

            if (char.IsDigit(c))
                return ReadNumber(pos);

            if (char.IsLetter(c) || c == '_')
                return ReadIdent(pos);

            throw new InvalidOperationException("bad char at " + pos);
        }

        public List<Token> All()
        {
            List<Token> list = new List<Token>();
            Token t;
            do
            {
                t = Next();
                list.Add(t);
            } while (t.Kind != TokenKind.Eof);
            return list;
        }

        void SkipSpace()
        {
            while (_i < _s.Length && char.IsWhiteSpace(_s[_i]))
                _i++;
        }

        Token ReadIdent(int pos)
        {
            int start = _i;
            _i++;
            while (_i < _s.Length && (char.IsLetterOrDigit(_s[_i]) || _s[_i] == '_'))
                _i++;
            return new Token(TokenKind.Ident, _s.Substring(start, _i - start), pos);
        }

        Token ReadNumber(int pos)
        {
            int start = _i;
            while (_i < _s.Length && char.IsDigit(_s[_i]))
                _i++;
            if (_i < _s.Length && _s[_i] == '.')
            {
                _i++;
                while (_i < _s.Length && char.IsDigit(_s[_i]))
                    _i++;
            }
            return new Token(TokenKind.Number, _s.Substring(start, _i - start), pos);
        }

        Token ReadString(char q, int pos)
        {
            _i++;
            StringBuilder sb = new StringBuilder();
            while (_i < _s.Length)
            {
                char c = _s[_i++];
                if (c == q)
                    return new Token(TokenKind.String, sb.ToString(), pos);
                if (c == '\\' && _i < _s.Length)
                {
                    sb.Append(_s[_i++]);
                    continue;
                }
                sb.Append(c);
            }
            throw new InvalidOperationException("unterminated string");
        }
    }
}
