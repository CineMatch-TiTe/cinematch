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
}: PartyMemberCardProps) {
  const initials = member.username?.substring(0, 2).toUpperCase() || '??'

  return (
    <div className="flex items-center justify-between p-3 rounded-xl bg-card border shadow-sm">
      <div className="flex items-center gap-3">
        <Avatar>
          <AvatarImage src={`https://api.dicebear.com/7.x/avataaars/svg?seed=${member.username}`} />
          <AvatarFallback>{initials}</AvatarFallback>
        </Avatar>
        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <span className="font-semibold">{member.username}</span>
            {member.is_leader && (
              <Badge variant="default" className="gap-1 text-xs px-2 py-0">
                <Crown size={12} className="fill-current" /> Leader
              </Badge>
            )}
            {isCurrentUser && (
              <Badge variant="outline" className="text-xs px-2 py-0">
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
            className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-100 hover:bg-green-100 dark:hover:bg-green-900 border-none"
          >
            Ready
          </Badge>
        )}

        {isCurrentUserLeader && !member.is_leader && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8">
                <MoreVertical className="h-4 w-4" />
                <span className="sr-only">Open menu</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => onPromote(member.user_id)}>
                <UserCheck className="mr-2 h-4 w-4" />
                <span>Promote to Leader</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={() => onKick(member.user_id)}
                className="text-red-600 focus:text-red-600"
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
