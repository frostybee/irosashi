/**
 * Fuse.js v7.1.0 - Lightweight fuzzy-search (http://fusejs.io)
 *
 * Copyright (c) 2025 Kiro Risk (http://kiro.me)
 * All Rights Reserved. Apache Software License 2.0
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 */
var e,t;e=this,t=function(){"use strict";function e(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var r=Object.getOwnPropertySymbols(e);t&&(r=r.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,r)}return n}function t(t){for(var n=1;n<arguments.length;n++){var r=null!=arguments[n]?arguments[n]:{};n%2?e(Object(r),!0).forEach((function(e){o(t,e,r[e])})):Object.getOwnPropertyDescriptors?Object.defineProperties(t,Object.getOwnPropertyDescriptors(r)):e(Object(r)).forEach((function(e){Object.defineProperty(t,e,Object.getOwnPropertyDescriptor(r,e))}))}return t}function n(e){return n="function"==typeof Symbol&&"symbol"==typeof Symbol.iterator?function(e){return typeof e}:function(e){return e&&"function"==typeof Symbol&&e.constructor===Symbol&&e!==Symbol.prototype?"symbol":typeof e},n(e)}function r(e,t){if(!(e instanceof t))throw new TypeError("Cannot call a class as a function")}function i(e,t){for(var n=0;n<t.length;n++){var r=t[n];r.enumerable=r.enumerable||!1,r.configurable=!0,"value"in r&&(r.writable=!0),Object.defineProperty(e,v(r.key),r)}}function u(e,t,n){return t&&i(e.prototype,t),n&&i(e,n),Object.defineProperty(e,"prototype",{writable:!1}),e}function o(e,t,n){return(t=v(t))in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function c(e,t){if("function"!=typeof t&&null!==t)throw new TypeError("Super expression must either be null or a function");e.prototype=Object.create(t&&t.prototype,{constructor:{value:e,writable:!0,configurable:!0}}),Object.defineProperty(e,"prototype",{writable:!1}),t&&s(e,t)}function a(e){return a=Object.setPrototypeOf?Object.getPrototypeOf.bind():function(e){return e.__proto__||Object.getPrototypeOf(e)},a(e)}function s(e,t){return s=Object.setPrototypeOf?Object.setPrototypeOf.bind():function(e,t){return e.__proto__=t,e},s(e,t)}function h(e,t){if(t&&("object"==typeof t||"function"==typeof t))return t;if(void 0!==t)throw new TypeError("Derived constructors may only return object or undefined");return function(e){if(void 0===e)throw new ReferenceError("this hasn't been initialised - super() hasn't been called");return e}(e)}function l(e){var t=function(){if("undefined"==typeof Reflect||!Reflect.construct)return!1;if(Reflect.construct.sham)return!1;if("function"==typeof Proxy)return!0;try{return Boolean.prototype.valueOf.call(Reflect.construct(Boolean,[],(function(){}))),!0}catch(e){return!1}}();return function(){var n,r=a(e);if(t){var i=a(this).constructor;n=Reflect.construct(r,arguments,i)}else n=r.apply(this,arguments);return h(this,n)}}function f(e){return function(e){if(Array.isArray(e))return d(e)}(e)||function(e){if("undefined"!=typeof Symbol&&null!=e[Symbol.iterator]||null!=e["@@iterator"])return Array.from(e)}(e)||function(e,t){if(e){if("string"==typeof e)return d(e,t);var n=Object.prototype.toString.call(e).slice(8,-1);return"Object"===n&&e.constructor&&(n=e.constructor.name),"Map"===n||"Set"===n?Array.from(e):"Arguments"===n||/^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)?d(e,t):void 0}}(e)||function(){throw new TypeError("Invalid attempt to spread non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method.")}()}function d(e,t){(null==t||t>e.length)&&(t=e.length);for(var n=0,r=new Array(t);n<t;n++)r[n]=e[n];return r}function v(e){var t=function(e,t){if("object"!=typeof e||null===e)return e;var n=e[Symbol.toPrimitive];if(void 0!==n){var r=n.call(e,t||"default");if("object"!=typeof r)return r;throw new TypeError("@@toPrimitive must return a primitive value.")}return("string"===t?String:Number)(e)}(e,"string");return"symbol"==typeof t?t:String(t)}function g(e){return Array.isArray?Array.isArray(e):"[object Array]"===M(e)}var y=1/0;function p(e){return null==e?"":function(e){if("string"==typeof e)return e;var t=e+"";return"0"==t&&1/e==-y?"-0":t}(e)}function A(e){return"string"==typeof e}function m(e){return"number"==typeof e}function C(e){return!0===e||!1===e||function(e){return k(e)&&null!==e}(e)&&"[object Boolean]"==M(e)}function k(e){return"object"===n(e)}function E(e){return null!=e}function F(e){return!e.trim().length}function M(e){return null==e?void 0===e?"[object Undefined]":"[object Null]":Object.prototype.toString.call(e)}var b=function(e){return"Missing ".concat(e," property in key")},D=function(e){return"Property 'weight' in key '".concat(e,"' must be a positive integer")},B=Object.prototype.hasOwnProperty,x=function(){function e(t){var n=this;r(this,e),this._keys=[],this._keyMap={};var i=0;t.forEach((function(e){var t=w(e);n._keys.push(t),n._keyMap[t.id]=t,i+=t.weight})),this._keys.forEach((function(e){e.weight/=i}))}return u(e,[{key:"get",value:function(e){return this._keyMap[e]}},{key:"keys",value:function(){return this._keys}},{key:"toJSON",value:function(){return JSON.stringify(this._keys)}}]),e}();function w(e){var t=null,n=null,r=null,i=1,u=null;if(A(e)||g(e))r=e,t=S(e),n=L(e);else{if(!B.call(e,"name"))throw new Error(b("name"));var o=e.name;if(r=o,B.call(e,"weight")&&(i=e.weight)<=0)throw new Error(D(o));t=S(o),n=L(o),u=e.getFn}return{path:t,id:n,weight:i,src:r,getFn:u}}function S(e){return g(e)?e:e.split(".")}function L(e){return g(e)?e.join("."):e}var _={useExtendedSearch:!1,getFn:function(e,t){var n=[],r=!1;return function e(t,i,u){if(E(t))if(i[u]){var o=t[i[u]];if(!E(o))return;if(u===i.length-1&&(A(o)||m(o)||C(o)))n.push(p(o));else if(g(o)){r=!0;for(var c=0,a=o.length;c<a;c+=1)e(o[c],i,u+1)}else i.length&&e(o,i,u+1)}else n.push(t)}(e,A(t)?t.split("."):t,0),r?n:n[0]},ignoreLocation:!1,ignoreFieldNorm:!1,fieldNormWeight:1},O=t(t(t(t({},{isCaseSensitive:!1,ignoreDiacritics:!1,includeScore:!1,keys:[],shouldSort:!0,sortFn:function(e,t){return e.score===t.score?e.idx<t.idx?-1:1:e.score<t.score?-1:1}}),{includeMatches:!1,findAllMatches:!1,minMatchCharLength:1}),{location:0,threshold:.6,distance:100}),_),j=/[^ ]+/g,I=function(){function e(){var t=arguments.length>0&&void 0!==arguments[0]?arguments[0]:{},n=t.getFn,i=void 0===n?O.getFn:n,u=t.fieldNormWeight,o=void 0===u?O.fieldNormWeight:u;r(this,e),this.norm=function(){var e=arguments.length>0&&void 0!==arguments[0]?arguments[0]:1,t=arguments.length>1&&void 0!==arguments[1]?arguments[1]:3,n=new Map,r=Math.pow(10,t);return{get:function(t){var i=t.match(j).length;if(n.has(i))return n.get(i);var u=1/Math.pow(i,.5*e),o=parseFloat(Math.round(u*r)/r);return n.set(i,o),o},clear:function(){n.clear()}}}(o,3),this.getFn=i,this.isCreated=!1,this.setIndexRecords()}return u(e,[{key:"setSources",value:function(){var e=arguments.length>0&&void 0!==arguments[0]?arguments[0]:[];this.docs=e}},{key:"setIndexRecords",value:function(){var e=arguments.length>0&&void 0!==arguments[0]?arguments[0]:[];this.records=e}},{key:"setKeys",value:function(){var e=this,t=arguments.length>0&&void 0!==arguments[0]?arguments[0]:[];this.keys=t,this._keysMap={},t.forEach((function(t,n){e._keysMap[t.id]=n}))}},{key:"create",value:function(){var e=this;!this.isCreated&&this.docs.length&&(this.isCreated=!0,A(this.docs[0])?this.docs.forEach((function(t,n){e._addString(t,n)})):this.docs.forEach((function(t,n){e._addObject(t,n)})),this.norm.clear())}},{key:"add",value:function(e){var t=this.size();A(e)?this._addString(e,t):this._addObject(e,t)}},{key:"removeAt",value:function(e){this.records.splice(e,1);for(var t=e,n=this.size();t<n;t+=1)this.records[t].i-=1}},{key:"getValueForItemAtKeyId",value:function(e,t){return e[this._keysMap[t]]}},{key:"size",value:function(){return this.records.length}},{key:"_addString",value:function(e,t){if(E(e)&&!F(e)){var n={v:e,i:t,n:this.norm.get(e)};this.records.push(n)}}},{key:"_addObject",value:function(e,t){var n=this,r={i:t,$:{}};this.keys.forEach((function(t,i){var u=t.getFn?t.getFn(e):n.getFn(e,t.path);if(E(u))if(g(u)){for(var o=[],c=[{nestedArrIndex:-1,value:u}];c.length;){var a=c.pop(),s=a.nestedArrIndex,h=a.value;if(E(h))if(A(h)&&!F(h)){var l={v:h,i:s,n:n.norm.get(h)};o.push(l)}else g(h)&&h.forEach((function(e,t){c.push({nestedArrIndex:t,value:e})}))}r.$[i]=o}else if(A(u)&&!F(u)){var f={v:u,n:n.norm.get(u)};r.$[i]=f}})),this.records.push(r)}},{key:"toJSON",value:function(){return{keys:this.keys,records:this.records}}}]),e}();function $(e,t){var n=arguments.length>2&&void 0!==arguments[2]?arguments[2]:{},r=n.getFn,i=void 0===r?O.getFn:r,u=n.fieldNormWeight,o=void 0===u?O.fieldNormWeight:u,c=new I({getFn:i,fieldNormWeight:o});return c.setKeys(e.map(w)),c.setSources(t),c.create(),c}function R(e){var t=arguments.length>1&&void 0!==arguments[1]?arguments[1]:{},n=t.errors,r=void 0===n?0:n,i=t.currentLocation,u=void 0===i?0:i,o=t.expectedLocation,c=void 0===o?0:o,a=t.distance,s=void 0===a?O.distance:a,h=t.ignoreLocation,l=void 0===h?O.ignoreLocation:h,f=r/e.length;if(l)return f;var d=Math.abs(c-u);return s?f+d/s:d?1:f}var N=32;function P(e,t,n){var r=arguments.length>3&&void 0!==arguments[3]?arguments[3]:{},i=r.location,u=void 0===i?O.location:i,o=r.distance,c=void 0===o?O.distance:o,a=r.threshold,s=void 0===a?O.threshold:a,h=r.findAllMatches,l=void 0===h?O.findAllMatches:h,f=r.minMatchCharLength,d=void 0===f?O.minMatchCharLength:f,v=r.includeMatches,g=void 0===v?O.includeMatches:v,y=r.ignoreLocation,p=void 0===y?O.ignoreLocation:y;if(t.length>N)throw new Error("Pattern length exceeds max of ".concat(N,"."));for(var A,m=t.length,C=e.length,k=Math.max(0,Math.min(u,C)),E=s,F=k,M=d>1||g,b=M?Array(C):[];(A=e.indexOf(t,F))>-1;){var D=R(t,{currentLocation:A,expectedLocation:k,distance:c,ignoreLocation:p});if(E=Math.min(D,E),F=A+m,M)for(var B=0;B<m;)b[A+B]=1,B+=1}F=-1;for(var x=[],w=1,S=m+C,L=1<<m-1,_=0;_<m;_+=1){for(var j=0,I=S;j<I;)R(t,{errors:_,currentLocation:k+I,expectedLocation:k,distance:c,ignoreLocation:p})<=E?j=I:S=I,I=Math.floor((S-j)/2+j);S=I;var $=Math.max(1,k-I+1),P=l?C:Math.min(k+I,C)+m,W=Array(P+2);W[P+1]=(1<<_)-1;for(var z=P;z>=$;z-=1){var T=z-1,K=n[e.charAt(T)];if(M&&(b[T]=+!!K),W[z]=(W[z+1]<<1|1)&K,_&&(W[z]|=(x[z+1]|x[z])<<1|1|x[z+1]),W[z]&L&&(w=R(t,{errors:_,currentLocation:T,expectedLocation:k,distance:c,ignoreLocation:p}))<=E){if(E=w,(F=T)<=k)break;$=Math.max(1,2*k-F)}}if(R(t,{errors:_+1,currentLocation:k,expectedLocation:k,distance:c,ignoreLocation:p})>E)break;x=W}var q={isMatch:F>=0,score:Math.max(.001,w)};if(M){var J=function(){for(var e=arguments.length>0&&void 0!==arguments[0]?arguments[0]:[],t=arguments.length>1&&void 0!==arguments[1]?arguments[1]:O.minMatchCharLength,n=[],r=-1,i=-1,u=0,o=e.length;u<o;u+=1){var c=e[u];c&&-1===r?r=u:c||-1===r||((i=u-1)-r+1>=t&&n.push([r,i]),r=-1)}return e[u-1]&&u-r>=t&&n.push([r,u-1]),n}(b,d);J.length?g&&(q.indices=J):q.isMatch=!1}return q}function W(e){for(var t={},n=0,r=e.length;n<r;n+=1){var i=e.charAt(n);t[i]=(t[i]||0)|1<<r-n-1}return t}var z=String.prototype.normalize?function(e){return e.normalize("NFD").replace(/[\u0300-\u036F\u0483-\u0489\u0591-\u05BD\u05BF\u05C1\u05C2\u05C4\u05C5\u05C7\u0610-\u061A\u064B-\u065F\u0670\u06D6-\u06DC\u06DF-\u06E4\u06E7\u06E8\u06EA-\u06ED\u0711\u0730-\u074A\u07A6-\u07B0\u07EB-\u07F3\u07FD\u0816-\u0819\u081B-\u0823\u0825-\u0827\u0829-\u082D\u0859-\u085B\u08D3-\u08E1\u08E3-\u0903\u093A-\u093C\u093E-\u094F\u0951-\u0957\u0962\u0963\u0981-\u0983\u09BC\u09BE-\u09C4\u09C7\u09C8\u09CB-\u09CD\u09D7\u09E2\u09E3\u09FE\u0A01-\u0A03\u0A3C\u0A3E-\u0A42\u0A47\u0A48\u0A4B-\u0A4D\u0A51\u0A70\u0A71\u0A75\u0A81-\u0A83\u0ABC\u0ABE-\u0AC5\u0AC7-\u0AC9\u0ACB-\u0ACD\u0AE2\u0AE3\u0AFA-\u0AFF\u0B01-\u0B03\u0B3C\u0B3E-\u0B44\u0B47\u0B48\u0B4B-\u0B4D\u0B56\u0B57\u0B62\u0B63\u0B82\u0BBE-\u0BC2\u0BC6-\u0BC8\u0BCA-\u0BCD\u0BD7\u0C00-\u0C04\u0C3E-\u0C44\u0C46-\u0C48\u0C4A-\u0C4D\u0C55\u0C56\u0C62\u0C63\u0C81-\u0C83\u0CBC\u0CBE-\u0CC4\u0CC6-\u0CC8\u0CCA-\u0CCD\u0CD5\u0CD6\u0CE2\u0CE3\u0D00-\u0D03\u0D3B\u0D3C\u0D3E-\u0D44\u0D46-\u0D48\u0D4A-\u0D4D\u0D57\u0D62\u0D63\u0D82\u0D83\u0DCA\u0DCF-\u0DD4\u0DD6\u0DD8-\u0DDF\u0DF2\u0DF3\u0E31\u0E34-\u0E3A\u0E47-\u0E4E\u0EB1\u0EB4-\u0EB9\u0EBB\u0EBC\u0EC8-\u0ECD\u0F18\u0F19\u0F35\u0F37\u0F39\u0F3E\u0F3F\u0F71-\u0F84\u0F86\u0F87\u0F8D-\u0F97\u0F99-\u0FBC\u0FC6\u102B-\u103E\u1056-\u1059\u105E-\u1060\u1062-\u1064\u1067-\u106D\u1071-\u1074\u1082-\u108D\u108F\u109A-\u109D\u135D-\u135F\u1712-\u1714\u1732-\u1734\u1752\u1753\u1772\u1773\u17B4-\u17D3\u17DD\u180B-\u180D\u1885\u1886\u18A9\u1920-\u192B\u1930-\u193B\u1A17-\u1A1B\u1A55-\u1A5E\u1A60-\u1A7C\u1A7F\u1AB0-\u1ABE\u1B00-\u1B04\u1B34-\u1B44\u1B6B-\u1B73\u1B80-\u1B82\u1BA1-\u1BAD\u1BE6-\u1BF3\u1C24-\u1C37\u1CD0-\u1CD2\u1CD4-\u1CE8\u1CED\u1CF2-\u1CF4\u1CF7-\u1CF9\u1DC0-\u1DF9\u1DFB-\u1DFF\u20D0-\u20F0\u2CEF-\u2CF1\u2D7F\u2DE0-\u2DFF\u302A-\u302F\u3099\u309A\uA66F-\uA672\uA674-\uA67D\uA69E\uA69F\uA6F0\uA6F1\uA802\uA806\uA80B\uA823-\uA827\uA880\uA881\uA8B4-\uA8C5\uA8E0-\uA8F1\uA8FF\uA926-\uA92D\uA947-\uA953\uA980-\uA983\uA9B3-\uA9C0\uA9E5\uAA29-\uAA36\uAA43\uAA4C\uAA4D\uAA7B-\uAA7D\uAAB0\uAAB2-\uAAB4\uAAB7\uAAB8\uAABE\uAABF\uAAC1\uAAEB-\uAAEF\uAAF5\uAAF6\uABE3-\uABEA\uABEC\uABED\uFB1E\uFE00-\uFE0F\uFE20-\uFE2F]/g,"")}:function(e){return e},T=function(){function e(t){var n=this,i=arguments.length>1&&void 0!==arguments[1]?arguments[1]:{},u=i.location,o=void 0===u?O.location:u,c=i.threshold,a=void 0===c?O.threshold:c,s=i.distance,h=void 0===s?O.distance:s,l=i.includeMatches,f=void 0===l?O.includeMatches:l,d=i.findAllMatches,v=void 0===d?O.findAllMatches:d,g=i.minMatchCharLength,y=void 0===g?O.minMatchCharLength:g,p=i.isCaseSensitive,A=void 0===p?O.isCaseSensitive:p,m=i.ignoreDiacritics,C=void 0===m?O.ignoreDiacritics:m,k=i.ignoreLocation,E=void 0===k?O.ignoreLocation:k;if(r(this,e),this.options={location:o,threshold:a,distance:h,includeMatches:f,findAllMatches:v,minMatchCharLength:y,isCaseSensitive:A,ignoreDiacritics:C,ignoreLocation:E},t=A?t:t.toLowerCase(),t=C?z(t):t,this.pattern=t,this.chunks=[],this.pattern.length){var F=function(e,t){n.chunks.push({pattern:e,alphabet:W(e),startIndex:t})},M=this.pattern.length;if(M>N){for(var b=0,D=M%N,B=M-D;b<B;)F(this.pattern.substr(b,N),b),b+=N;if(D){var x=M-N;F(this.pattern.substr(x),x)}}else F(this.pattern,0)}}return u(e,[{key:"searchIn",value:function(e){var t=this.options,n=t.isCaseSensitive,r=t.ignoreDiacritics,i=t.includeMatches;if(e=n?e:e.toLowerCase(),e=r?z(e):e,this.pattern===e){var u={isMatch:!0,score:0};return i&&(u.indices=[[0,e.length-1]]),u}var o=this.options,c=o.location,a=o.distance,s=o.threshold,h=o.findAllMatches,l=o.minMatchCharLength,d=o.ignoreLocation,v=[],g=0,y=!1;this.chunks.forEach((function(t){var n=t.pattern,r=t.alphabet,u=t.startIndex,o=P(e,n,r,{location:c+u,distance:a,threshold:s,findAllMatches:h,minMatchCharLength:l,includeMatches:i,ignoreLocation:d}),p=o.isMatch,A=o.score,m=o.indices;p&&(y=!0),g+=A,p&&m&&(v=[].concat(f(v),f(m)))}));var p={isMatch:y,score:y?g/this.chunks.length:1};return y&&i&&(p.indices=v),p}}]),e}(),K=function(){function e(t){r(this,e),this.pattern=t}return u(e,[{key:"search",value:function(){}}],[{key:"isMultiMatch",value:function(e){return q(e,this.multiRegex)}},{key:"isSingleMatch",value:function(e){return q(e,this.singleRegex)}}]),e}();function q(e,t){var n=e.match(t);return n?n[1]:null}var J=function(e){c(n,e);var t=l(n);function n(e){return r(this,n),t.call(this,e)}return u(n,[{key:"search",value:function(e){var t=e===this.pattern;return{isMatch:t,score:t?0:1,indices:[0,this.pattern.length-1]}}}],[{key:"type",get:function(){return"exact"}},{key:"multiRegex",get:function(){return/^="(.*)"$/}},{key:"singleRegex",get:function(){return/^=(.*)$/}}]),n}(K),U=function(e){c(n,e);var t=l(n);function n(e){return r(this,n),t.call(this,e)}return u(n,[{key:"search",value:function(e){var t=-1===e.indexOf(this.pattern);return{isMatch:t,score:t?0:1,indices:[0,e.length-1]}}}],[{key:"type",get:function(){return"inverse-exact"}},{key:"multiRegex",get:function(){return/^!"(.*)"$/}},{key:"singleRegex",get:function(){return/^!(.*)$/}}]),n}(K),V=function(e){c(n,e);var t=l(n);function n(e){return r(this,n),t.call(this,e)}return u(n,[{key:"search",value:function(e){var t=e.startsWith(this.pattern);return{isMatch:t,score:t?0:1,indices:[0,this.pattern.length-1]}}}],[{key:"type",get:function(){return"prefix-exact"}},{key:"multiRegex",get:function(){return/^\^"(.*)"$/}},{key:"singleRegex",get:function(){return/^\^(.*)$/}}]),n}(K),G=function(e){c(n,e);var t=l(n);function n(e){return r(this,n),t.call(this,e)}return u(n,[{key:"search",value:function(e){var t=!e.startsWith(this.pattern);return{isMatch:t,score:t?0:1,indices:[0,e.length-1]}}}],[{key:"type",get:function(){return"inverse-prefix-exact"}},{key:"multiRegex",get:function(){return/^!\^"(.*)"$/}},{key:"singleRegex",get:function(){return/^!\^(.*)$/}}]),n}(K),H=function(e){c(n,e);var t=l(n);function n(e){return r(this,n),t.call(this,e)}return u(n,[{key:"search",value:function(e){var t=e.endsWith(this.pattern);return{isMatch:t,score:t?0:1,indices:[e.length-this.pattern.length,e.length-1]}}}],[{key:"type",get:function(){return"suffix-exact"}},{key:"multiRegex",get:function(){return/^"(.*)"\$$/}},{key:"singleRegex",get:function(){return/^(.*)\$$/}}]),n}(K),Q=function(e){c(n,e);var t=l(n);function n(e){return r(this,n),t.call(this,e)}return u(n,[{key:"search",value:function(e){var t=!e.endsWith(this.pattern);return{isMatch:t,score:t?0:1,indices:[0,e.length-1]}}}],[{key:"type",get:function(){return"inverse-suffix-exact"}},{key:"multiRegex",get:function(){return/^!"(.*)"\$$/}},{key:"singleRegex",get:function(){return/^!(.*)\$$/}}]),n}(K),X=function(e){c(n,e);var t=l(n);function n(e){var i,u=arguments.length>1&&void 0!==arguments[1]?arguments[1]:{},o=u.location,c=void 0===o?O.location:o,a=u.threshold,s=void 0===a?O.threshold:a,h=u.distance,l=void 0===h?O.distance:h,f=u.includeMatches,d=void 0===f?O.includeMatches:f,v=u.findAllMatches,g=void 0===v?O.findAllMatches:v,y=u.minMatchCharLength,p=void 0===y?O.minMatchCharLength:y,A=u.isCaseSensitive,m=void 0===A?O.isCaseSensitive:A,C=u.ignoreDiacritics,k=void 0===C?O.ignoreDiacritics:C,E=u.ignoreLocation,F=void 0===E?O.ignoreLocation:E;return r(this,n),(i=t.call(this,e))._bitapSearch=new T(e,{location:c,threshold:s,distance:l,includeMatches:d,findAllMatches:g,minMatchCharLength:p,isCaseSensitive:m,ignoreDiacritics:k,ignoreLocation:F}),i}return u(n,[{key:"search",value:function(e){return this._bitapSearch.searchIn(e)}}],[{key:"type",get:function(){return"fuzzy"}},{key:"multiRegex",get:function(){return/^"(.*)"$/}},{key:"singleRegex",get:function(){return/^(.*)$/}}]),n}(K),Y=function(e){c(n,e);var t=l(n);function n(e){return r(this,n),t.call(this,e)}return u(n,[{key:"search",value:function(e){for(var t,n=0,r=[],i=this.pattern.length;(t=e.indexOf(this.pattern,n))>-1;)n=t+i,r.push([t,n-1]);var u=!!r.length;return{isMatch:u,score:u?0:1,indices:r}}}],[{key:"type",get:function(){return"include"}},{key:"multiRegex",get:function(){return/^'"(.*)"$/}},{key:"singleRegex",get:function(){return/^'(.*)$/}}]),n}(K),Z=[J,Y,V,G,Q,H,U,X],ee=Z.length,te=/ +(?=(?:[^\"]*\"[^\"]*\")*[^\"]*$)/,ne=new Set([X.type,Y.type]),re=function(){function e(t){var n=arguments.length>1&&void 0!==arguments[1]?arguments[1]:{},i=n.isCaseSensitive,u=void 0===i?O.isCaseSensitive:i,o=n.ignoreDiacritics,c=void 0===o?O.ignoreDiacritics:o,a=n.includeMatches,s=void 0===a?O.includeMatches:a,h=n.minMatchCharLength,l=void 0===h?O.minMatchCharLength:h,f=n.ignoreLocation,d=void 0===f?O.ignoreLocation:f,v=n.findAllMatches,g=void 0===v?O.findAllMatches:v,y=n.location,p=void 0===y?O.location:y,A=n.threshold,m=void 0===A?O.threshold:A,C=n.distance,k=void 0===C?O.distance:C;r(this,e),this.query=null,this.options={isCaseSensitive:u,ignoreDiacritics:c,includeMatches:s,minMatchCharLength:l,findAllMatches:g,ignoreLocation:d,location:p,threshold:m,distance:k},t=u?t:t.toLowerCase(),t=c?z(t):t,this.pattern=t,this.query=function(e){var t=arguments.length>1&&void 0!==arguments[1]?arguments[1]:{};return e.split("|").map((function(e){for(var n=e.trim().split(te).filter((function(e){return e&&!!e.trim()})),r=[],i=0,u=n.length;i<u;i+=1){for(var o=n[i],c=!1,a=-1;!c&&++a<ee;){var s=Z[a],h=s.isMultiMatch(o);h&&(r.push(new s(h,t)),c=!0)}if(!c)for(a=-1;++a<ee;){var l=Z[a],f=l.isSingleMatch(o);if(f){r.push(new l(f,t));break}}}return r}))}(this.pattern,this.options)}return u(e,[{key:"searchIn",value:function(e){var t=this.query;if(!t)return{isMatch:!1,score:1};var n=this.options,r=n.includeMatches,i=n.isCaseSensitive,u=n.ignoreDiacritics;e=i?e:e.toLowerCase(),e=u?z(e):e;for(var o=0,c=[],a=0,s=0,h=t.length;s<h;s+=1){var l=t[s];c.length=0,o=0;for(var d=0,v=l.length;d<v;d+=1){var g=l[d],y=g.search(e),p=y.isMatch,A=y.indices,m=y.score;if(!p){a=0,o=0,c.length=0;break}if(o+=1,a+=m,r){var C=g.constructor.type;ne.has(C)?c=[].concat(f(c),f(A)):c.push(A)}}if(o){var k={isMatch:!0,score:a/o};return r&&(k.indices=c),k}}return{isMatch:!1,score:1}}}],[{key:"condition",value:function(e,t){return t.useExtendedSearch}}]),e}(),ie=[];function ue(e,t){for(var n=0,r=ie.length;n<r;n+=1){var i=ie[n];if(i.condition(e,t))return new i(e,t)}return new T(e,t)}var oe="$and",ce="$or",ae="$path",se="$val",he=function(e){return!(!e[oe]&&!e[ce])},le=function(e){return o({},oe,Object.keys(e).map((function(t){return o({},t,e[t])})))};function fe(e,t){var n=(arguments.length>2&&void 0!==arguments[2]?arguments[2]:{}).auto,r=void 0===n||n;return he(e)||(e=le(e)),function e(n){var i=Object.keys(n),u=function(e){return!!e[ae]}(n);if(!u&&i.length>1&&!he(n))return e(le(n));if(function(e){return!g(e)&&k(e)&&!he(e)}(n)){var o=u?n[ae]:i[0],c=u?n[se]:n[o];if(!A(c))throw new Error(function(e){return"Invalid value for key ".concat(e)}(o));var a={keyId:L(o),pattern:c};return r&&(a.searcher=ue(c,t)),a}var s={children:[],operator:i[0]};return i.forEach((function(t){var r=n[t];g(r)&&r.forEach((function(t){s.children.push(e(t))}))})),s}(e)}function de(e,t){var n=e.matches;t.matches=[],E(n)&&n.forEach((function(e){if(E(e.indices)&&e.indices.length){var n={indices:e.indices,value:e.value};e.key&&(n.key=e.key.src),e.idx>-1&&(n.refIndex=e.idx),t.matches.push(n)}}))}function ve(e,t){t.score=e.score}var ge=function(){function e(n){var i=arguments.length>1&&void 0!==arguments[1]?arguments[1]:{},u=arguments.length>2?arguments[2]:void 0;r(this,e),this.options=t(t({},O),i),this.options.useExtendedSearch,this._keyStore=new x(this.options.keys),this.setCollection(n,u)}return u(e,[{key:"setCollection",value:function(e,t){if(this._docs=e,t&&!(t instanceof I))throw new Error("Incorrect 'index' type");this._myIndex=t||$(this.options.keys,this._docs,{getFn:this.options.getFn,fieldNormWeight:this.options.fieldNormWeight})}},{key:"add",value:function(e){E(e)&&(this._docs.push(e),this._myIndex.add(e))}},{key:"remove",value:function(){for(var e=arguments.length>0&&void 0!==arguments[0]?arguments[0]:function(){return!1},t=[],n=0,r=this._docs.length;n<r;n+=1){var i=this._docs[n];e(i,n)&&(this.removeAt(n),n-=1,r-=1,t.push(i))}return t}},{key:"removeAt",value:function(e){this._docs.splice(e,1),this._myIndex.removeAt(e)}},{key:"getIndex",value:function(){return this._myIndex}},{key:"search",value:function(e){var t=(arguments.length>1&&void 0!==arguments[1]?arguments[1]:{}).limit,n=void 0===t?-1:t,r=this.options,i=r.includeMatches,u=r.includeScore,o=r.shouldSort,c=r.sortFn,a=r.ignoreFieldNorm,s=A(e)?A(this._docs[0])?this._searchStringList(e):this._searchObjectList(e):this._searchLogical(e);return function(e,t){var n=t.ignoreFieldNorm,r=void 0===n?O.ignoreFieldNorm:n;e.forEach((function(e){var t=1;e.matches.forEach((function(e){var n=e.key,i=e.norm,u=e.score,o=n?n.weight:null;t*=Math.pow(0===u&&o?Number.EPSILON:u,(o||1)*(r?1:i))})),e.score=t}))}(s,{ignoreFieldNorm:a}),o&&s.sort(c),m(n)&&n>-1&&(s=s.slice(0,n)),function(e,t){var n=arguments.length>2&&void 0!==arguments[2]?arguments[2]:{},r=n.includeMatches,i=void 0===r?O.includeMatches:r,u=n.includeScore,o=void 0===u?O.includeScore:u,c=[];return i&&c.push(de),o&&c.push(ve),e.map((function(e){var n=e.idx,r={item:t[n],refIndex:n};return c.length&&c.forEach((function(t){t(e,r)})),r}))}(s,this._docs,{includeMatches:i,includeScore:u})}},{key:"_searchStringList",value:function(e){var t=ue(e,this.options),n=this._myIndex.records,r=[];return n.forEach((function(e){var n=e.v,i=e.i,u=e.n;if(E(n)){var o=t.searchIn(n),c=o.isMatch,a=o.score,s=o.indices;c&&r.push({item:n,idx:i,matches:[{score:a,value:n,norm:u,indices:s}]})}})),r}},{key:"_searchLogical",value:function(e){var t=this,n=fe(e,this.options),r=function e(n,r,i){if(!n.children){var u=n.keyId,o=n.searcher,c=t._findMatches({key:t._keyStore.get(u),value:t._myIndex.getValueForItemAtKeyId(r,u),searcher:o});return c&&c.length?[{idx:i,item:r,matches:c}]:[]}for(var a=[],s=0,h=n.children.length;s<h;s+=1){var l=e(n.children[s],r,i);if(l.length)a.push.apply(a,f(l));else if(n.operator===oe)return[]}return a},i=this._myIndex.records,u={},o=[];return i.forEach((function(e){var t=e.$,i=e.i;if(E(t)){var c=r(n,t,i);c.length&&(u[i]||(u[i]={idx:i,item:t,matches:[]},o.push(u[i])),c.forEach((function(e){var t,n=e.matches;(t=u[i].matches).push.apply(t,f(n))})))}})),o}},{key:"_searchObjectList",value:function(e){var t=this,n=ue(e,this.options),r=this._myIndex,i=r.keys,u=r.records,o=[];return u.forEach((function(e){var r=e.$,u=e.i;if(E(r)){var c=[];i.forEach((function(e,i){c.push.apply(c,f(t._findMatches({key:e,value:r[i],searcher:n})))})),c.length&&o.push({idx:u,item:r,matches:c})}})),o}},{key:"_findMatches",value:function(e){var t=e.key,n=e.value,r=e.searcher;if(!E(n))return[];var i=[];if(g(n))n.forEach((function(e){var n=e.v,u=e.i,o=e.n;if(E(n)){var c=r.searchIn(n),a=c.isMatch,s=c.score,h=c.indices;a&&i.push({score:s,key:t,value:n,idx:u,norm:o,indices:h})}}));else{var u=n.v,o=n.n,c=r.searchIn(u),a=c.isMatch,s=c.score,h=c.indices;a&&i.push({score:s,key:t,value:u,norm:o,indices:h})}return i}}]),e}();return ge.version="7.1.0",ge.createIndex=$,ge.parseIndex=function(e){var t=arguments.length>1&&void 0!==arguments[1]?arguments[1]:{},n=t.getFn,r=void 0===n?O.getFn:n,i=t.fieldNormWeight,u=void 0===i?O.fieldNormWeight:i,o=e.keys,c=e.records,a=new I({getFn:r,fieldNormWeight:u});return a.setKeys(o),a.setIndexRecords(c),a},ge.config=O,function(){ie.push.apply(ie,arguments)}(re),ge},"object"==typeof exports&&"undefined"!=typeof module?module.exports=t():"function"==typeof define&&define.amd?define(t):(e="undefined"!=typeof globalThis?globalThis:e||self).Fuse=t();
// Telescope: command-palette page navigation. Opens with Ctrl+/ (Cmd+/ on
// Mac), fuzzy-searches the build-time page index with Fuse.js, and keeps
// pinned and recently visited pages in localStorage (paths only; titles and
// descriptions are always re-resolved from the fetched index, so stored data
// self-heals when pages move or disappear).
(function () {
    'use strict';

    const sarde = window.__SARDE__ || {};
    const cfg = (sarde.pluginConfig && sarde.pluginConfig.telescope) || {};

    const FALLBACK = {
        trigger: 'Quick navigation',
        tab_search: 'Search',
        tab_recent: 'Recent',
        placeholder: 'Search pages...',
        filter_recent: 'Filter recent pages...',
        pin: 'Pin page',
        unpin: 'Unpin page',
        clear: 'Clear',
        clear_pinned: 'Clear pinned pages',
        clear_recent: 'Clear recent pages',
        no_recent: 'No recently visited pages',
        no_results: 'No pages found matching your search',
        close: 'Close',
        loading: 'Loading pages...',
        pinned_section: 'Pinned Pages',
        recent_section: 'Recently Visited',
        results_section: 'Search Results',
        results_count: '{count} results found',
        showing_of: 'Showing {shown} of {total} results. Refine your search to see more.',
        dialog_opened: 'Search dialog opened',
        recent_cleared: 'Recent pages cleared',
        pinned_cleared: 'Pinned pages cleared',
        hint_navigate: 'navigate',
        hint_select: 'select',
        hint_pin: 'pin',
        hint_close: 'close'
    };

    // Server-resolved i18n label, falling back to English when the key is
    // missing or came through unresolved (raw "telescope.*" key).
    function str(key) {
        const v = cfg.strings && cfg.strings[key];
        if (typeof v === 'string' && v !== '' && v.indexOf('telescope.') !== 0) return v;
        return FALLBACK[key] || key;
    }

    const SHORTCUT_KEY = (typeof cfg.shortcut_key === 'string' && cfg.shortcut_key.length > 0) ? cfg.shortcut_key : '/';
    const MAX_RESULTS = cfg.max_results > 0 ? cfg.max_results : 20;
    const MAX_RECENT = cfg.max_recent > 0 ? cfg.max_recent : 10;
    const MAX_PINNED = cfg.max_pinned > 0 ? cfg.max_pinned : 50;
    const DEBOUNCE_MS = (cfg.debounce_ms != null && cfg.debounce_ms >= 0) ? cfg.debounce_ms : 120;
    const DEFAULT_TAB = cfg.default_tab === 'recent' ? 'recent' : 'search';
    const RECENT_STORAGE_CAP = 50;

    const SITE_ID = sarde.siteId || 'site';
    const KEY_PINNED = 'telescope:' + SITE_ID + ':pinned';
    const KEY_RECENT = 'telescope:' + SITE_ID + ':recent';

    const BASE = (sarde.basePath || '/').replace(/\/+$/, '') + '/';
    const LANG = sarde.lang || '';

    // ── State ──
    let allPages = [];
    let byPath = new Map();
    let filteredPages = [];
    let searchResultsWithMatches = [];
    let searchQuery = '';
    let selectedIndex = 0;
    let currentTab = DEFAULT_TAB;
    let fuseInstance = null;
    let isLoading = false;
    let fetchDone = false;
    let fetchInProgress = null;
    let hasMouseMovedSinceOpen = false;
    let isInNavigationMode = false;
    let debounceTimeout = null;

    // ── localStorage (path arrays only) ──
    function loadPaths(key) {
        try {
            const raw = localStorage.getItem(key);
            const parsed = raw ? JSON.parse(raw) : [];
            return Array.isArray(parsed) ? parsed.filter(function (p) { return typeof p === 'string'; }) : [];
        } catch (_e) {
            return [];
        }
    }
    function savePaths(key, paths) {
        try {
            localStorage.setItem(key, JSON.stringify(paths));
        } catch (_e) { /* storage full or unavailable */ }
    }

    let recentPaths = loadPaths(KEY_RECENT);
    let pinnedPaths = loadPaths(KEY_PINNED);

    function normalizePath(path) {
        if (!path) return '/';
        // Pretty URLs end with a slash; leave file-like paths (with an
        // extension) alone.
        if (path.charAt(path.length - 1) !== '/' && path.lastIndexOf('.') <= path.lastIndexOf('/')) {
            return path + '/';
        }
        return path;
    }

    function recordVisit(path) {
        path = normalizePath(path);
        recentPaths = recentPaths.filter(function (p) { return p !== path; });
        recentPaths.unshift(path);
        recentPaths = recentPaths.slice(0, RECENT_STORAGE_CAP);
        savePaths(KEY_RECENT, recentPaths);
    }

    // Stored paths are only trusted once the index is loaded; unknown paths
    // (deleted pages, other sites on the same origin) are dropped.
    function validPages(paths) {
        if (allPages.length === 0) return [];
        const out = [];
        for (let i = 0; i < paths.length; i++) {
            const page = byPath.get(paths[i]);
            if (page) out.push(page);
        }
        return out;
    }

    function isPinned(path) {
        return pinnedPaths.indexOf(path) !== -1;
    }

    function togglePin(path) {
        const idx = pinnedPaths.indexOf(path);
        if (idx > -1) {
            pinnedPaths.splice(idx, 1);
        } else {
            if (pinnedPaths.length >= MAX_PINNED) pinnedPaths.shift();
            pinnedPaths.push(path);
        }
        savePaths(KEY_PINNED, pinnedPaths);
        renderSearchResults();
        renderRecentResults();
    }

    function clearRecent() {
        recentPaths = [];
        try { localStorage.removeItem(KEY_RECENT); } catch (_e) { /* ignore */ }
        renderSearchResults();
        renderRecentResults();
        announce(str('recent_cleared'));
    }

    function clearPinned() {
        pinnedPaths = [];
        try { localStorage.removeItem(KEY_PINNED); } catch (_e) { /* ignore */ }
        renderSearchResults();
        renderRecentResults();
        announce(str('pinned_cleared'));
    }

    // ── Index fetch (lazy, on first open) ──
    function ensurePages() {
        if (fetchDone) return Promise.resolve();
        if (fetchInProgress) return fetchInProgress;
        isLoading = true;
        updateLoadingState();
        fetchInProgress = fetch(BASE + 'telescope-pages.json')
            .then(function (response) {
                if (!response.ok) throw new Error('telescope index fetch failed: ' + response.status);
                return response.json();
            })
            .then(function (entries) {
                if (!Array.isArray(entries)) entries = [];
                allPages = entries.filter(function (e) {
                    return e && typeof e.path === 'string' && (!LANG || !e.lang || e.lang === LANG);
                });
                byPath = new Map();
                allPages.forEach(function (e) { byPath.set(e.path, e); });

                // Prune stored paths the index no longer knows.
                const validRecent = recentPaths.filter(function (p) { return byPath.has(p); });
                if (validRecent.length !== recentPaths.length) {
                    recentPaths = validRecent;
                    savePaths(KEY_RECENT, recentPaths);
                }
                const validPinned = pinnedPaths.filter(function (p) { return byPath.has(p); });
                if (validPinned.length !== pinnedPaths.length) {
                    pinnedPaths = validPinned;
                    savePaths(KEY_PINNED, pinnedPaths);
                }

                if (typeof Fuse !== 'undefined' && allPages.length > 0) {
                    fuseInstance = new Fuse(allPages, {
                        keys: [
                            { name: 'title', weight: 1.0 },
                            { name: 'path', weight: 0.6 },
                            { name: 'tags', weight: 0.5 },
                            { name: 'collection', weight: 0.4 },
                            { name: 'description', weight: 0.3 }
                        ],
                        threshold: 0.3,
                        ignoreLocation: true,
                        distance: 100,
                        minMatchCharLength: 2,
                        findAllMatches: false,
                        useExtendedSearch: true,
                        includeScore: true,
                        includeMatches: true
                    });
                }
                fetchDone = true;
            })
            .catch(function (_err) {
                allPages = [];
                byPath = new Map();
            })
            .finally(function () {
                isLoading = false;
                fetchInProgress = null;
                updateLoadingState();
            });
        return fetchInProgress;
    }

    // ── DOM helpers ──
    function $(id) { return document.getElementById(id); }

    function escapeHtml(text) {
        const map = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' };
        return String(text).replace(/[&<>"']/g, function (ch) { return map[ch]; });
    }

    function announce(message) {
        const live = $('sarde-telescope-live');
        if (!live) return;
        live.textContent = '';
        requestAnimationFrame(function () { live.textContent = message; });
    }

    function updateLoadingState() {
        const loadingEl = $('sarde-telescope-loading');
        const resultsEl = $('sarde-telescope-results');
        if (loadingEl) loadingEl.style.display = isLoading ? 'flex' : 'none';
        if (resultsEl) resultsEl.style.display = isLoading ? 'none' : 'block';
    }

    // ── Modal ──
    function modalHTML() {
        return '<dialog id="sarde-telescope-dialog" class="sarde-telescope" aria-label="' + escapeHtml(str('trigger')) + '">'
            + '<div class="sarde-telescope-modal">'
            + '<button type="button" class="sarde-telescope-close" id="sarde-telescope-close" aria-label="' + escapeHtml(str('close')) + '">'
            + '<svg aria-hidden="true" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12" stroke-linecap="round"/></svg>'
            + '</button>'
            + '<div class="sarde-telescope-tabs" role="tablist">'
            + '<button type="button" class="sarde-telescope-tab" data-tab="search" role="tab" aria-selected="false" id="sarde-telescope-tab-search" aria-controls="sarde-telescope-search-section">' + escapeHtml(str('tab_search')) + '</button>'
            + '<button type="button" class="sarde-telescope-tab" data-tab="recent" role="tab" aria-selected="false" id="sarde-telescope-tab-recent" aria-controls="sarde-telescope-recent-section">' + escapeHtml(str('tab_recent')) + '</button>'
            + '</div>'
            + '<div class="sarde-telescope-search-header">'
            + '<input id="sarde-telescope-input" class="sarde-telescope-input" type="text" role="combobox" aria-expanded="true" aria-autocomplete="list" aria-haspopup="listbox" aria-controls="sarde-telescope-results" aria-activedescendant="" placeholder="' + escapeHtml(str('placeholder')) + '" autocomplete="off" spellcheck="false">'
            + '</div>'
            + '<div id="sarde-telescope-search-section" class="sarde-telescope-section" role="tabpanel" aria-labelledby="sarde-telescope-tab-search">'
            + '<div id="sarde-telescope-loading" class="sarde-telescope-loading"><div class="sarde-telescope-spinner"></div><span>' + escapeHtml(str('loading')) + '</span></div>'
            + '<ul id="sarde-telescope-results" class="sarde-telescope-results" role="listbox" aria-label="' + escapeHtml(str('results_section')) + '"></ul>'
            + '</div>'
            + '<div id="sarde-telescope-recent-section" class="sarde-telescope-section" role="tabpanel" aria-labelledby="sarde-telescope-tab-recent">'
            + '<ul id="sarde-telescope-recent-results" class="sarde-telescope-results" role="listbox" aria-label="' + escapeHtml(str('tab_recent')) + '"></ul>'
            + '</div>'
            + '<div id="sarde-telescope-live" class="sarde-telescope-sr-only" aria-live="polite" aria-atomic="true"></div>'
            + '<div class="sarde-telescope-footer"><div class="sarde-telescope-shortcuts">'
            + '<span><kbd>↑</kbd><kbd>↓</kbd> ' + escapeHtml(str('hint_navigate')) + '</span>'
            + '<span><kbd>↵</kbd> ' + escapeHtml(str('hint_select')) + '</span>'
            + '<span><kbd>Space</kbd> ' + escapeHtml(str('hint_pin')) + '</span>'
            + '<span><kbd>Esc</kbd> ' + escapeHtml(str('hint_close')) + '</span>'
            + '</div></div>'
            + '</div></dialog>';
    }

    let modalReady = false;
    function ensureModal() {
        if (modalReady && $('sarde-telescope-dialog')) return;
        if (!$('sarde-telescope-dialog')) {
            document.body.insertAdjacentHTML('beforeend', modalHTML());
        }
        bindModalListeners();
        modalReady = true;
    }

    function bindModalListeners() {
        const input = $('sarde-telescope-input');
        if (input) {
            input.addEventListener('input', function (event) {
                if (debounceTimeout) clearTimeout(debounceTimeout);
                debounceTimeout = setTimeout(function () { handleSearchInput(event); }, DEBOUNCE_MS);
            });
            input.addEventListener('keydown', handleSearchKeyDown);
        }
        const closeBtn = $('sarde-telescope-close');
        if (closeBtn) closeBtn.addEventListener('click', close);

        const dialog = $('sarde-telescope-dialog');
        if (dialog) {
            dialog.addEventListener('click', function (event) {
                const tab = event.target.closest('.sarde-telescope-tab');
                if (tab && tab.dataset.tab) {
                    switchTab(tab.dataset.tab);
                    return;
                }
                // Backdrop click: the dialog element itself, outside the panel.
                if (event.target === dialog) close();
            });
            // Native close (Esc via the browser) keeps state consistent.
            dialog.addEventListener('close', onDialogClosed);
        }
    }

    function getActiveResultsContainer() {
        return currentTab === 'recent' ? $('sarde-telescope-recent-results') : $('sarde-telescope-results');
    }

    // ── Search ──
    function getMatchesForPage(page) {
        for (let i = 0; i < searchResultsWithMatches.length; i++) {
            if (searchResultsWithMatches[i].item.path === page.path) {
                return searchResultsWithMatches[i].matches;
            }
        }
        return undefined;
    }

    function highlightMatches(text, matches, key) {
        if (!matches || !text) return escapeHtml(text || '');
        let match = null;
        for (let i = 0; i < matches.length; i++) {
            if (matches[i].key === key) { match = matches[i]; break; }
        }
        if (!match || !match.indices || match.indices.length === 0) return escapeHtml(text);

        let result = '';
        let lastIndex = 0;
        const sorted = match.indices.slice().sort(function (a, b) { return a[0] - b[0]; });
        for (let i = 0; i < sorted.length; i++) {
            const start = sorted[i][0];
            const end = sorted[i][1];
            if (start > lastIndex) result += escapeHtml(text.slice(lastIndex, start));
            if (start >= lastIndex) {
                result += '<mark class="sarde-telescope-highlight">' + escapeHtml(text.slice(start, end + 1)) + '</mark>';
                lastIndex = end + 1;
            }
        }
        result += escapeHtml(text.slice(lastIndex));
        return result;
    }

    function filterPages() {
        const query = searchQuery.trim();
        const lowerQuery = query.toLowerCase();

        if (!query) {
            filteredPages = allPages.slice();
            searchResultsWithMatches = [];
        } else if (fuseInstance) {
            const results = fuseInstance.search(query);
            // Exact title > title prefix > word start > raw Fuse score, so
            // quick-navigation matches beat deep fuzzy matches.
            results.sort(function (a, b) {
                const aTitle = (a.item.title || '').toLowerCase();
                const bTitle = (b.item.title || '').toLowerCase();
                const aExact = aTitle === lowerQuery;
                const bExact = bTitle === lowerQuery;
                if (aExact && !bExact) return -1;
                if (bExact && !aExact) return 1;
                const aPrefix = aTitle.indexOf(lowerQuery) === 0;
                const bPrefix = bTitle.indexOf(lowerQuery) === 0;
                if (aPrefix && !bPrefix) return -1;
                if (bPrefix && !aPrefix) return 1;
                const aWord = aTitle.split(/\s+/).some(function (w) { return w.indexOf(lowerQuery) === 0; });
                const bWord = bTitle.split(/\s+/).some(function (w) { return w.indexOf(lowerQuery) === 0; });
                if (aWord && !bWord) return -1;
                if (bWord && !aWord) return 1;
                return (a.score || 0) - (b.score || 0);
            });
            searchResultsWithMatches = results;
            filteredPages = results.map(function (r) { return r.item; });
        } else {
            filteredPages = allPages.filter(function (page) {
                return (page.title || '').toLowerCase().indexOf(lowerQuery) !== -1
                    || (page.path || '').toLowerCase().indexOf(lowerQuery) !== -1
                    || (page.description || '').toLowerCase().indexOf(lowerQuery) !== -1;
            });
        }

        selectedIndex = 0;
        renderSearchResults();
        updateSelectedResult();

        const count = filteredPages.length;
        if (count === 0) {
            announce(str('no_results'));
        } else if (count > MAX_RESULTS) {
            announce(str('showing_of').replace('{shown}', MAX_RESULTS).replace('{total}', count));
        } else {
            announce(str('results_count').replace('{count}', count));
        }
    }

    function handleSearchInput(event) {
        isInNavigationMode = false;
        searchQuery = event.target.value;

        if (currentTab === 'recent') {
            const lower = searchQuery.toLowerCase();
            filteredPages = validPages(recentPaths).filter(function (page) {
                return (page.title || '').toLowerCase().indexOf(lower) !== -1
                    || (page.path || '').toLowerCase().indexOf(lower) !== -1
                    || (page.description || '').toLowerCase().indexOf(lower) !== -1;
            });
            selectedIndex = 0;
            renderRecentResults();
            updateSelectedResult();
        } else {
            filterPages();
        }
    }

    // ── Keyboard ──
    function getKeyCode(key) {
        if (/^[a-zA-Z]$/.test(key)) return 'Key' + key.toUpperCase();
        if (/^[0-9]$/.test(key)) return 'Digit' + key;
        const special = {
            '/': 'Slash', '\\': 'Backslash', '-': 'Minus', '=': 'Equal',
            '[': 'BracketLeft', ']': 'BracketRight', ';': 'Semicolon',
            "'": 'Quote', ',': 'Comma', '.': 'Period', '`': 'Backquote', ' ': 'Space'
        };
        return special[key] || null;
    }

    function handleGlobalKeyDown(event) {
        const expectedCode = getKeyCode(SHORTCUT_KEY);
        const viaCharacter = event.key === SHORTCUT_KEY;
        const viaCode = expectedCode !== null && event.code === expectedCode;
        // Either Ctrl or Cmd opens the palette. For character matches shift is
        // ignored (some layouts need shift to type the character); for
        // physical-code matches shift must be up.
        const shiftOk = viaCharacter ? true : !event.shiftKey;
        if ((viaCharacter || viaCode) && (event.ctrlKey || event.metaKey) && shiftOk && !event.altKey) {
            event.preventDefault();
            event.stopPropagation();
            open();
            return;
        }
        const dialog = $('sarde-telescope-dialog');
        if (event.key === 'Escape' && dialog && dialog.open) {
            event.preventDefault();
            close();
        }
    }

    function handleSearchKeyDown(event) {
        const dialog = $('sarde-telescope-dialog');
        if (!dialog || !dialog.open) return;

        switch (event.key) {
            case 'Escape':
                event.preventDefault();
                close();
                break;
            case 'ArrowDown':
                event.preventDefault();
                isInNavigationMode = true;
                navigateResults(1);
                break;
            case 'ArrowUp':
                event.preventDefault();
                isInNavigationMode = true;
                navigateResults(-1);
                break;
            case 'Enter':
                event.preventDefault();
                selectCurrentItem();
                break;
            case ' ': {
                // Space pins when it cannot be intended as typed text.
                const input = $('sarde-telescope-input');
                if (input && (input.value === '' || input.selectionStart === 0 || isInNavigationMode)) {
                    event.preventDefault();
                    togglePinForSelectedItem();
                }
                break;
            }
            default:
                break;
        }
    }

    function navigateResults(direction) {
        const container = getActiveResultsContainer();
        if (!container) return;
        const totalItems = container.querySelectorAll('.sarde-telescope-result').length;
        if (totalItems === 0) return;
        selectedIndex = (selectedIndex + direction + totalItems) % totalItems;
        updateSelectedResult();
    }

    function updateSelectedResult() {
        const container = getActiveResultsContainer();
        if (!container) return;
        const items = container.querySelectorAll('.sarde-telescope-result');
        items.forEach(function (item) {
            item.classList.remove('sarde-telescope-result-selected');
            item.setAttribute('aria-selected', 'false');
        });
        const selected = container.querySelector('[data-index="' + selectedIndex + '"]');
        if (!selected) return;
        selected.classList.add('sarde-telescope-result-selected');
        selected.setAttribute('aria-selected', 'true');
        const input = $('sarde-telescope-input');
        if (input) input.setAttribute('aria-activedescendant', selected.id);
        selected.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }

    function selectCurrentItem() {
        const selected = document.querySelector('.sarde-telescope-result-selected');
        if (selected && selected.hasAttribute('data-path')) {
            navigateToPage(selected.getAttribute('data-path'));
        }
    }

    function togglePinForSelectedItem() {
        const selected = document.querySelector('.sarde-telescope-result-selected');
        if (selected && selected.hasAttribute('data-path')) {
            togglePin(selected.getAttribute('data-path'));
        }
    }

    function navigateToPage(path) {
        recordVisit(path);
        close();
        window.location.href = path;
    }

    // ── Rendering ──
    function createSectionHeader(text, ariaClearLabel, onClear) {
        const header = document.createElement('li');
        header.className = 'sarde-telescope-section-separator';
        header.setAttribute('role', 'presentation');
        const title = document.createElement('span');
        title.textContent = text;
        header.appendChild(title);
        if (onClear) {
            const clearBtn = document.createElement('button');
            clearBtn.type = 'button';
            clearBtn.className = 'sarde-telescope-clear-btn';
            clearBtn.textContent = str('clear');
            clearBtn.setAttribute('aria-label', ariaClearLabel);
            clearBtn.addEventListener('click', function (e) {
                e.stopPropagation();
                onClear();
            });
            header.appendChild(clearBtn);
        }
        return header;
    }

    function createNoResults(message) {
        const li = document.createElement('li');
        li.className = 'sarde-telescope-no-results';
        li.setAttribute('role', 'presentation');
        li.textContent = message;
        return li;
    }

    function createResultItem(page, index, containerId) {
        const li = document.createElement('li');
        li.id = containerId + '-option-' + index;
        li.className = 'sarde-telescope-result' + (index === selectedIndex ? ' sarde-telescope-result-selected' : '');
        li.setAttribute('role', 'option');
        li.setAttribute('aria-selected', index === selectedIndex ? 'true' : 'false');
        li.setAttribute('data-index', String(index));
        li.setAttribute('data-path', page.path);

        const pinned = isPinned(page.path);
        const pinButton = document.createElement('button');
        pinButton.type = 'button';
        pinButton.className = 'sarde-telescope-pin-btn' + (pinned ? ' sarde-telescope-pin-btn-pinned' : '');
        pinButton.innerHTML = '<svg aria-hidden="true" viewBox="0 0 24 24" width="14" height="14"><path d="M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z"></path></svg>';
        pinButton.title = pinned ? str('unpin') : str('pin');
        pinButton.setAttribute('aria-label', pinned ? str('unpin') : str('pin'));
        pinButton.addEventListener('click', function (event) {
            event.stopPropagation();
            event.preventDefault();
            togglePin(page.path);
        });

        const contentRow = document.createElement('div');
        contentRow.className = 'sarde-telescope-result-content';

        const matches = searchQuery.trim() ? getMatchesForPage(page) : undefined;

        const titleDiv = document.createElement('div');
        titleDiv.className = 'sarde-telescope-result-title';
        titleDiv.innerHTML = highlightMatches(page.title || '', matches, 'title');
        contentRow.appendChild(titleDiv);

        if (page.description) {
            const descDiv = document.createElement('div');
            descDiv.className = 'sarde-telescope-result-description';
            descDiv.innerHTML = highlightMatches(page.description, matches, 'description');
            contentRow.appendChild(descDiv);
        }

        if (page.collection) {
            const collectionSpan = document.createElement('span');
            collectionSpan.className = 'sarde-telescope-result-collection';
            collectionSpan.textContent = page.collection;
            contentRow.appendChild(collectionSpan);
        }

        if (page.tags && page.tags.length > 0) {
            const tagsDiv = document.createElement('div');
            tagsDiv.className = 'sarde-telescope-result-tags';
            page.tags.forEach(function (tag) {
                const span = document.createElement('span');
                span.className = 'sarde-telescope-tag';
                span.textContent = tag;
                tagsDiv.appendChild(span);
            });
            contentRow.appendChild(tagsDiv);
        }

        li.appendChild(contentRow);
        li.appendChild(pinButton);

        li.addEventListener('click', function () {
            navigateToPage(page.path);
        });
        li.addEventListener('mouseenter', function () {
            // Ignore hover until the mouse actually moves, so a modal opening
            // under the cursor cannot steal the selection.
            if (!hasMouseMovedSinceOpen) return;
            selectedIndex = index;
            updateSelectedResult();
        });

        return li;
    }

    function renderSearchResults() {
        const container = $('sarde-telescope-results');
        if (!container) return;
        container.innerHTML = '';

        const validPinned = validPages(pinnedPaths);
        const validRecent = validPages(recentPaths);

        let index = 0;

        if (validPinned.length > 0) {
            container.appendChild(createSectionHeader(str('pinned_section'), str('clear_pinned'), clearPinned));
            validPinned.forEach(function (page) {
                container.appendChild(createResultItem(page, index, 'sarde-telescope-results'));
                index++;
            });
        }

        const nonPinnedRecent = validRecent.filter(function (p) { return !isPinned(p.path); }).slice(0, MAX_RECENT);
        if (nonPinnedRecent.length > 0) {
            container.appendChild(createSectionHeader(str('recent_section'), str('clear_recent'), clearRecent));
            nonPinnedRecent.forEach(function (page) {
                container.appendChild(createResultItem(page, index, 'sarde-telescope-results'));
                index++;
            });
        }

        // Keep search results free of rows already shown above.
        const shownPaths = {};
        validPinned.forEach(function (p) { shownPaths[p.path] = true; });
        nonPinnedRecent.forEach(function (p) { shownPaths[p.path] = true; });
        const results = filteredPages.filter(function (p) { return !shownPaths[p.path]; });

        if (index > 0 && results.length > 0) {
            container.appendChild(createSectionHeader(str('results_section')));
        }

        if (results.length === 0 && index === 0) {
            container.appendChild(createNoResults(str('no_results')));
            return;
        }

        const display = results.slice(0, MAX_RESULTS);
        display.forEach(function (page) {
            container.appendChild(createResultItem(page, index, 'sarde-telescope-results'));
            index++;
        });

        if (results.length > MAX_RESULTS) {
            const indicator = document.createElement('li');
            indicator.className = 'sarde-telescope-more-results';
            indicator.setAttribute('role', 'presentation');
            indicator.textContent = str('showing_of').replace('{shown}', MAX_RESULTS).replace('{total}', results.length);
            container.appendChild(indicator);
        }
    }

    function renderRecentResults() {
        const container = $('sarde-telescope-recent-results');
        if (!container) return;
        container.innerHTML = '';

        const pages = searchQuery.trim() ? filteredPages : validPages(recentPaths).slice(0, MAX_RECENT);
        if (pages.length === 0) {
            container.appendChild(createNoResults(searchQuery.trim() ? str('no_results') : str('no_recent')));
            return;
        }
        pages.forEach(function (page, index) {
            container.appendChild(createResultItem(page, index, 'sarde-telescope-recent-results'));
        });
    }

    function switchTab(tabName) {
        currentTab = tabName === 'recent' ? 'recent' : 'search';
        selectedIndex = 0;
        searchQuery = '';
        const input = $('sarde-telescope-input');
        if (input) {
            input.value = '';
            input.placeholder = currentTab === 'recent' ? str('filter_recent') : str('placeholder');
        }
        filteredPages = allPages.slice();

        document.querySelectorAll('.sarde-telescope-tab').forEach(function (tab) {
            const active = tab.dataset.tab === currentTab;
            tab.classList.toggle('sarde-telescope-tab-active', active);
            tab.setAttribute('aria-selected', active ? 'true' : 'false');
        });
        document.querySelectorAll('.sarde-telescope-section').forEach(function (section) {
            section.classList.toggle('sarde-telescope-section-active', section.id === 'sarde-telescope-' + currentTab + '-section');
        });
        if (input) input.setAttribute('aria-controls', 'sarde-telescope-' + (currentTab === 'recent' ? 'recent-results' : 'results'));

        if (currentTab === 'recent') {
            renderRecentResults();
        } else {
            renderSearchResults();
        }
        updateSelectedResult();
    }

    // ── Open / close ──
    function open() {
        ensureModal();
        const dialog = $('sarde-telescope-dialog');
        if (!dialog || dialog.open) return;

        searchQuery = '';
        selectedIndex = 0;
        filteredPages = allPages.slice();
        searchResultsWithMatches = [];
        isInNavigationMode = false;

        dialog.showModal();

        dialog.classList.add('sarde-telescope-pointer-disabled');
        hasMouseMovedSinceOpen = false;
        dialog.addEventListener('mousemove', function () {
            hasMouseMovedSinceOpen = true;
            dialog.classList.remove('sarde-telescope-pointer-disabled');
        }, { once: true });

        updateLoadingState();
        switchTab(DEFAULT_TAB);

        ensurePages().then(function () {
            if (!dialog.open) return;
            filteredPages = allPages.slice();
            if (currentTab === 'recent') {
                renderRecentResults();
            } else {
                renderSearchResults();
            }
            updateSelectedResult();
        });

        requestAnimationFrame(function () {
            const input = $('sarde-telescope-input');
            if (input) {
                input.value = '';
                input.focus();
            }
        });

        announce(str('dialog_opened'));
    }

    function onDialogClosed() {
        if (debounceTimeout) {
            clearTimeout(debounceTimeout);
            debounceTimeout = null;
        }
        const input = $('sarde-telescope-input');
        if (input) input.setAttribute('aria-activedescendant', '');
    }

    function close() {
        const dialog = $('sarde-telescope-dialog');
        if (!dialog || !dialog.open) return;
        dialog.close();
        onDialogClosed();
    }

    // ── Trigger button ──
    function createTrigger() {
        const actions = document.querySelector('.sarde-header-actions');
        if (!actions) return;

        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'sarde-telescope-trigger';
        button.setAttribute('aria-label', str('trigger'));
        button.title = str('trigger');
        button.setAttribute('aria-keyshortcuts', 'Control+' + SHORTCUT_KEY);
        button.innerHTML = '<svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">'
            + '<path d="m10.065 12.493-6.18 1.318a.934.934 0 0 1-1.108-.702l-.537-2.15a1.07 1.07 0 0 1 .691-1.265l13.504-4.44"/>'
            + '<path d="m13.56 11.747 4.332-.924"/>'
            + '<path d="m16 21-3.105-6.21"/>'
            + '<path d="M16.485 5.94a2 2 0 0 1 1.455-2.425l1.09-.272a1 1 0 0 1 1.212.727l1.515 6.06a1 1 0 0 1-.727 1.213l-1.09.272a2 2 0 0 1-2.425-1.455z"/>'
            + '<path d="m6.158 8.633 1.114 4.456"/>'
            + '<path d="m8 21 3.105-6.21"/>'
            + '</svg>'
            + '<kbd class="sarde-telescope-shortcut"><kbd class="sarde-telescope-shortcut-mod">Ctrl</kbd><kbd>' + escapeHtml(SHORTCUT_KEY) + '</kbd></kbd>';

        const isMac = (navigator.userAgentData && navigator.userAgentData.platform === 'macOS')
            || /Mac|iPhone|iPod|iPad/i.test(navigator.userAgent);
        if (isMac) {
            const mod = button.querySelector('.sarde-telescope-shortcut-mod');
            if (mod) mod.textContent = '⌘';
            button.setAttribute('aria-keyshortcuts', 'Meta+' + SHORTCUT_KEY);
        }

        button.addEventListener('click', open);

        // Sit next to the built-in search trigger when present, otherwise
        // lead the header actions.
        const searchTrigger = actions.querySelector('.sarde-search-trigger');
        if (searchTrigger) {
            searchTrigger.insertAdjacentElement('afterend', button);
        } else {
            actions.insertAdjacentElement('afterbegin', button);
        }
    }

    // ── Init ──
    function init() {
        recordVisit(window.location.pathname);
        createTrigger();
        document.addEventListener('keydown', handleGlobalKeyDown, true);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
