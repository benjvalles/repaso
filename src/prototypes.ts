declare global {
  interface String {
    getIdNumber(): number
    toInt(): number
    toFloat<T>(d?: number, str?: boolean, trimZeros?: boolean, grouping?: boolean): T
  }

  interface Number {
    toInt(): number
    toFloat<T>(decimals?: number, toStr?: boolean, trimZeros?: boolean, grouping?: boolean): T
  }
  
  interface NumberConstructor {
    DECIMAL_SEP: string
    GROUP_SEP: string
    LOCALE: string
  }
}

export {}

Number.prototype.toInt = String.prototype.toInt = function (this: string) { return parseInt(this) || 0 }

 
function roundToPrecision(numStr: string, decimals: number): number {
  const negative = numStr.startsWith('-')
  const [intS, fracS = ''] = (negative ? numStr.slice(1) : numStr).split('.')
  const padded = (fracS + '0'.repeat(decimals + 1)).slice(0, decimals + 1)
  const keep = padded.slice(0, decimals)
  const next = parseInt(padded[decimals], 10)
  const scaled = BigInt((intS || '0') + keep)
  const rounded = next >= 5 ? scaled + 1n : scaled
  const roundedStr = rounded.toString().padStart(decimals + 1, '0')
  const newInt = roundedStr.slice(0, -decimals || roundedStr.length) || '0'
  const newFrac = decimals > 0 ? roundedStr.slice(-decimals) : ''
  const result = newFrac ? `${newInt}.${newFrac}` : newInt
  return parseFloat((negative ? '-' : '') + result)
}

/**
 * In JavaScript all numbers are {@link http://en.wikipedia.org/wiki/IEEE_754 IEEE 754} floating point numbers.
 * Solucionado con redondeo mediante {@link roundToPrecision} que opera con aritmética de enteros
 * para evitar el drift de coma flotante.
 * #### Historia de esta función
 * Una forma de solucionarlo es multiplicar el valor * 100 y dividir por 100, pero esto no es fiable en todos los casos:
 * (7.165).toFixed(2) da 7.17 pero (8.165).toFixed(2) da 8.16
 * 03/12/2025: detectado número *2044.1567 * 100 / 100* daba *2044.1567000000002* por lo que las
 * formulaciones anteriores fallaban en un u otro caso.
 * 02/03/2026: detectado número *0.43 * 0.43* daba 0.18489999999999998
 * @param decimals Número de decimales
 * @param toStr Convertir a string
 * @param trimZeros Quita ceros del final de la coma (solo si toStr es true)
 * @param grouping Añade separador de miles (solo si toStr es true)
 */
Number.prototype.toFloat = String.prototype.toFloat = function <T>(
  this: string,
  decimals = 16,
  toStr = false,
  trimZeros = false,
  grouping = false
): T {
  const str = this.toString()
    .replace('Infinity', '0')
    .replace(new RegExp(Number.DECIMAL_SEP, 'g'), '.')
    .split('.')
    .splice(0, 2)
    .join('.')
  const [intPart, fracPart = ''] = str.split('.')
  const sourceDecimals = fracPart.length
  const significantDigits = 15
  const intDigits = intPart.replace('-', '').replace(/^0+/, '').length || 1
  const maxMeaningfulDecimals = Math.max(0, significantDigits - intDigits)
  const effectiveDecimals = Math.min(decimals, sourceDecimals, maxMeaningfulDecimals, 16)
  const num = roundToPrecision(str, effectiveDecimals)
  if (toStr)
    return <unknown>Number(isNaN(num) ? 0 : num).toLocaleString(Number.LOCALE, {
      useGrouping: grouping,
      minimumFractionDigits: trimZeros ? 0 : decimals,
      maximumFractionDigits: decimals
    }) as T
  return <unknown>(isNaN(num) ? 0 : num) as T
}
Number.prototype.toInt = String.prototype.toInt = function (this: string) { return parseInt(this) || 0 }

Number.DECIMAL_SEP = ','
Number.GROUP_SEP = '.'
Number.LOCALE = 'es-ES'