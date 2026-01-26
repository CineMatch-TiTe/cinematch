'use client'

import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { MemberInfo } from '@/model'
import { Crown, MoreVertical, UserX, UserCheck } from 'lucide-react'

interface PartyMemberCardProps {
  member: MemberInfo
  isCurrentUserLeader: boolean
  isCurrentUser: boolean
  onKick: (memberId: string) => void
  onPromote: (memberId: string) => void
}

export function PartyMemberCard({
  member,
  isCurrentUserLeader,
  isCurrentUser,
  onKick,
  onPromote
}: Readonly<PartyMemberCardProps>) {
  const initials = member.username?.substring(0, 2).toUpperCase() || '??'

  return (
    <div className="flex items-center justify-between p-3 rounded-xl bg-zinc-900/50 border border-zinc-800 shadow-sm backdrop-blur-sm">
      <div className="flex items-center gap-3">
        <Avatar className="border-2 border-zinc-800">
          <AvatarImage src={`https://api.dicebear.com/7.x/avataaars/svg?seed=${member.username}`} />
          <AvatarFallback className="bg-zinc-800 text-zinc-400">{initials}</AvatarFallback>
        </Avatar>
        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-zinc-200">{member.username}</span>
            {member.is_leader && (
              <Badge
                variant="default"
                className="gap-1 text-xs px-2 py-0 bg-yellow-500/20 text-yellow-500 hover:bg-yellow-500/30 border-yellow-500/20"
              >
                <Crown size={12} className="fill-current" /> Leader
              </Badge>
            )}
            {isCurrentUser && (
              <Badge variant="outline" className="text-xs px-2 py-0 border-zinc-700 text-zinc-500">
                You
              </Badge>
            )}
          </div>
          {/* Status indicator could go here (Ready/Not Ready) if needed later */}
        </div>
      </div>

      <div className="flex items-center gap-2">
        {member.is_ready && (
          <Badge
            variant="secondary"
            className="bg-green-500/20 text-green-400 border border-green-500/30"
          >
            Ready
          </Badge>
        )}

        {isCurrentUserLeader && !member.is_leader && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800"
              >
                <MoreVertical className="h-4 w-4" />
                <span className="sr-only">Open menu</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="bg-zinc-950 border-zinc-800 text-zinc-200">
              <DropdownMenuItem
                onClick={() => onPromote(member.user_id)}
                className="focus:bg-zinc-900 focus:text-zinc-100 cursor-pointer"
              >
                <UserCheck className="mr-2 h-4 w-4" />
                <span>Promote to Leader</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={() => onKick(member.user_id)}
                className="text-red-500 focus:text-red-400 focus:bg-red-500/10 cursor-pointer"
              >
                <UserX className="mr-2 h-4 w-4" />
                <span>Kick Member</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>
    </div>
  )
}
