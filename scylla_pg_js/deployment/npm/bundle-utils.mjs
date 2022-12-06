import {copyFileSync, existsSync, mkdirSync, readdirSync, statSync} from "fs"
import path from "path"

export function getCliArg(prop, required = true){
  const idx = process.argv.indexOf("--" + prop)
  const val = idx >= 0 ? process.argv[idx + 1] : undefined
  if(!val && required){
    throw new Error(`Required cli argument is not defined --${ prop }`)
  }
  return val
}

export function walkDir(rootdir, filter, dirFilter){
  const files = []

  const recursiveWalk = (dir) => {
    if(!existsSync(dir)){
      return
    }
    readdirSync(dir).forEach( f => {
      const dirPath = path.join(dir, f)
      const isDirectory = statSync(dirPath).isDirectory()
      if(isDirectory){
        if((!dirFilter || dirFilter(f))){
          recursiveWalk(dirPath)
        }
      }else{
        if(!filter || filter(f)){
          files.push(dirPath)
        }
      }
    })
  }
  recursiveWalk(rootdir)
  return files
}

export function copyFiles(srcDir, targetDir, filter, dirFilter){
  // copy other referenced files
  const allFiles = walkDir(srcDir, filter, dirFilter)
  for(const f of allFiles){
    const relative = path.relative(srcDir, f)
    const outFile = path.resolve(targetDir, relative)
    mkdirSync(path.dirname(outFile), { recursive: true })
    copyFileSync(f, outFile)
  }
}
