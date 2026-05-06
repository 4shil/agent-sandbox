import { useEffect, useRef } from 'react'
import { Terminal } from 'xterm'
import { FitAddon } from 'xterm-addon-fit'
import 'xterm/css/xterm.css'

interface TerminalProps {
  className?: string
}

export default function TerminalComponent({ className }: TerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null)
  const terminalInstance = useRef<Terminal | null>(null)

  useEffect(() => {
    if (!terminalRef.current) return

    const term = new Terminal({
      theme: {
        background: '#0a0a0f',
        foreground: '#dcdcff',
        cursor: '#64b4ff',
        selectionBackground: '#64b4ff80',
      },
      fontFamily: 'Courier New, Courier, monospace',
      fontSize: 14,
    })

    const fitAddon = new FitAddon()
    term.loadAddon(fitAddon)

    term.open(terminalRef.current)
    
    // Fit terminal to container
    setTimeout(() => {
      try {
        fitAddon.fit()
      } catch (e) {
        console.error('Failed to fit terminal:', e)
      }
    }, 100)

    terminalInstance.current = term

    // Write ASCII art header
    term.writeln('     _____')
    term.writeln('    /     \\')
    term.writeln('   /  _   \\')
    term.writeln('  /  / \\   \\')
    term.writeln(' /  /   \\   \\')
    term.writeln('/__/     \\___\\')
    term.writeln('')
    term.writeln('abox - Agent Sandbox Dashboard')
    term.writeln('')
    term.writeln('[TUI Dashboard will be embedded here]')
    term.writeln('')
    term.writeln('Press any key to launch TUI...')

    // Handle input to launch TUI (placeholder)
    term.onData(() => {
      term.writeln('Launching TUI...')
      // TODO: Invoke Tauri command to launch abox TUI
    })

    return () => {
      term.dispose()
    }
  }, [])

  return (
    <div className={`w-full h-full ${className || ''}`}>
      <div ref={terminalRef} className="w-full h-full" />
    </div>
  )
}
