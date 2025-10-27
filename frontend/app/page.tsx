"use client"

import { useCallback, useEffect, useMemo, useState } from "react"
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { RefreshCw } from "lucide-react"

const API_BASE = process.env.NEXT_PUBLIC_BACKEND_URL ?? "http://127.0.0.1:8787"

type HistoryResponseItem = {
  id: number
  natural: string
  sql: string
  executed: number
  created_at: string
}

type HistoryItem = {
  id: number
  natural: string
  sql: string
  executed: boolean
  createdAt: string
}

type GenerateSqlResponse = {
  generated_sql: string
  history_id: number
}

type ExecutionRow = Record<string, unknown>

const formatCellValue = (value: unknown) => {
  if (value === null || value === undefined) return "-"
  if (typeof value === "object") {
    try {
      return JSON.stringify(value)
    } catch {
      return "[object]"
    }
  }
  return String(value)
}

export default function Home() {
  const [dataRequest, setDataRequest] = useState("")
  const [generatedSQL, setGeneratedSQL] = useState("")
  const [currentId, setCurrentId] = useState<number | null>(null)
  const [isGenerating, setIsGenerating] = useState(false)
  const [isRunning, setIsRunning] = useState(false)
  const [generationError, setGenerationError] = useState<string | null>(null)
  const [executionError, setExecutionError] = useState<string | null>(null)
  const [historyError, setHistoryError] = useState<string | null>(null)
  const [historyLoading, setHistoryLoading] = useState(false)
  const [history, setHistory] = useState<HistoryItem[]>([])
  const [queryResult, setQueryResult] = useState<ExecutionRow[] | null>(null)
  const [queryMessage, setQueryMessage] = useState<string | null>(null)

  const fetchHistory = useCallback(async () => {
    setHistoryLoading(true)
    setHistoryError(null)
    try {
      const response = await fetch(`${API_BASE}/history`)
      if (!response.ok) {
        const message = await response.text()
        throw new Error(message || `History request failed (${response.status})`)
      }
      const data = (await response.json()) as HistoryResponseItem[]
      setHistory(
        data.map((item) => ({
          id: item.id,
          natural: item.natural,
          sql: item.sql,
          executed: item.executed === 1,
          createdAt: item.created_at,
        })),
      )
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to load history"
      setHistoryError(message)
    } finally {
      setHistoryLoading(false)
    }
  }, [])

  useEffect(() => {
    void fetchHistory()
  }, [fetchHistory])

  const handleGenerateSQL = useCallback(async () => {
    if (!dataRequest.trim()) {
      setGenerationError("Please describe the data you want to retrieve.")
      return
    }

    setIsGenerating(true)
    setGenerationError(null)
    setExecutionError(null)
    setQueryResult(null)
    setQueryMessage(null)

    try {
      const response = await fetch(`${API_BASE}/nl2sql`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ prompt: dataRequest }),
      })

      if (!response.ok) {
        const message = await response.text()
        throw new Error(message || `Generation failed (${response.status})`)
      }

      const data = (await response.json()) as GenerateSqlResponse
      setGeneratedSQL(data.generated_sql)
      setCurrentId(data.history_id)
      await fetchHistory()
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to generate SQL"
      setGenerationError(message)
    } finally {
      setIsGenerating(false)
    }
  }, [dataRequest, fetchHistory])

  const handleSqlEdit = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setGeneratedSQL(e.target.value);
    setCurrentId(null); 
  };

  const handleRunSQL = useCallback(async () => {
    if (!generatedSQL.trim()) {
      setExecutionError("Generate or paste SQL before running it.")
      return
    }

    setIsRunning(true)
    setExecutionError(null)
    setQueryResult(null)
    setQueryMessage(null)

    try {
      const response = await fetch(`${API_BASE}/execsql`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          history_id: currentId,
          sql_to_run: generatedSQL,
        }),
      })

      if (!response.ok) {
        const message = (await response.text()) || `Execution failed (${response.status})`

        if (response.status === 400 && message.startsWith("SQL Syntax Error:")) {
          setExecutionError(`We couldn't run the query because of a syntax issue:\n${message.replace("SQL Syntax Error: ", "")}`)
          return
        }

        if (response.status === 403) {
          setExecutionError(
            "The query was blocked for safety. Remove statements such as DROP, ALTER, TRUNCATE, or CREATE DATABASE.",
          )
          return
        }

        throw new Error(message)
      }

      const data = (await response.json()) as unknown
      if (Array.isArray(data)) {
        const rows = data as ExecutionRow[]
        setQueryResult(rows)
        setQueryMessage(
          rows.length ? `Returned ${rows.length} row${rows.length === 1 ? "" : "s"}.` : "Query executed successfully. No rows returned.",
        )
      } else {
        setQueryResult([])
        setQueryMessage("Query executed, but no structured rows were returned.")
      }

      await fetchHistory()
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to execute SQL"
      setExecutionError(message)
    } finally {
      setIsRunning(false)
    }
  }, [currentId, fetchHistory, generatedSQL])

  const resultColumns = useMemo(() => {
    if (!queryResult || queryResult.length === 0) return []
    const unique = new Set<string>()
    for (const row of queryResult) {
      Object.keys(row ?? {}).forEach((key) => unique.add(key))
    }
    return Array.from(unique)
  }, [queryResult])

  return (
    <div className="min-h-screen bg-background p-6 md:p-8">
      <div className="mx-auto max-w-4xl space-y-6">
        <h1 className="text-4xl font-bold text-foreground mb-8">AI SQL Generator & Runner</h1>

        <Card>
          <CardHeader>
            <CardTitle className="text-xl">Data Request</CardTitle>
          </CardHeader>
          <CardContent>
            <Textarea
              placeholder="Show me the 5 most recent orders"
              value={dataRequest}
              onChange={(e) => setDataRequest(e.target.value)}
              className="min-h-[120px] font-sans resize-none"
            />
          </CardContent>
          <CardFooter>
            <Button
              onClick={() => void handleGenerateSQL()}
              className="w-full md:w-auto"
              disabled={isGenerating || !dataRequest.trim()}
            >
              {isGenerating ? "Generating..." : "Generate SQL"}
            </Button>
          </CardFooter>
          {generationError && (
            <Alert variant="destructive" className="mx-6 mb-6">
              <AlertTitle>Failed to generate SQL</AlertTitle>
              <AlertDescription>{generationError}</AlertDescription>
            </Alert>
          )}
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-xl">
              Generated SQL {currentId !== null ? `(ID: ${currentId})` : ""}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Textarea
              value={generatedSQL}
              onChange={handleSqlEdit}
              placeholder="Generated SQL will appear here."
              className="min-h-[120px] font-mono text-sm bg-muted resize-none"
            />
          </CardContent>
          <CardFooter>
            <Button
              onClick={() => void handleRunSQL()}
              className="w-full md:w-auto"
              disabled={isRunning || !generatedSQL.trim()}
            >
              {isRunning ? "Running..." : "Run SQL"}
            </Button>
          </CardFooter>
        </Card>

        {executionError && (
          <Alert variant="destructive">
            <AlertTitle>SQL failed</AlertTitle>
            <AlertDescription>{executionError}</AlertDescription>
          </Alert>
        )}

        {queryMessage && (
          <Alert>
            <AlertTitle>Execution complete</AlertTitle>
            <AlertDescription>{queryMessage}</AlertDescription>
          </Alert>
        )}

        {queryResult && queryResult.length > 0 && resultColumns.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle className="text-xl">Execution Result</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full table-auto text-sm">
                  <thead>
                    <tr className="bg-muted text-left">
                      {resultColumns.map((column) => (
                        <th key={column} className="px-3 py-2 font-semibold text-foreground">
                          {column}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {queryResult.map((row, index) => (
                      <tr key={index} className="border-b border-border last:border-0">
                        {resultColumns.map((column) => (
                          <td key={column} className="px-3 py-2">
                            {formatCellValue(row?.[column])}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        )}

        {queryResult && queryResult.length === 0 && !executionError && (
          <Card>
            <CardHeader>
              <CardTitle className="text-xl">Execution Result</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground">No rows returned.</p>
            </CardContent>
          </Card>
        )}

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
            <CardTitle className="text-xl">Recent History</CardTitle>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => void fetchHistory()}
              className="h-8 w-8"
              disabled={historyLoading}
            >
              <RefreshCw className={`h-4 w-4 ${historyLoading ? "animate-spin" : ""}`} />
            </Button>
          </CardHeader>
          <CardContent>
            {historyError && (
              <Alert variant="destructive" className="mb-4">
                <AlertTitle>Failed to load history</AlertTitle>
                <AlertDescription>{historyError}</AlertDescription>
              </Alert>
            )}
            <div className="space-y-6">
              {history.length === 0 && !historyError && (
                <p className="text-sm text-muted-foreground">No history yet. Generate a query to get started.</p>
              )}
              {history.map((item) => (
                <div key={item.id} className="space-y-2">
                  <div className="flex items-start gap-3">
                    <Badge variant={item.executed ? "default" : "secondary"} className="mt-0.5">
                      {item.executed ? "[Executed]" : "[Not Executed]"}
                    </Badge>
                    <div className="flex-1">
                      <p className="text-sm text-foreground">{item.natural}</p>
                      <p className="text-xs text-muted-foreground">#{item.id}</p>
                    </div>
                  </div>
                  <div className="bg-muted rounded-md p-3 border border-border">
                    <pre className="text-xs font-mono text-foreground overflow-x-auto">
                      <code>{item.sql}</code>
                    </pre>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
