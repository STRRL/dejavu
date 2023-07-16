"use client"
import { IconButton, InputAdornment, InputBase, Paper, TextField } from '@mui/material'
import SearchIcon from '@mui/icons-material/Search';
import { useState } from 'react';
import { useRouter } from 'next/navigation'

export default function Nav() {
    const [searchText, setSearchText] = useState('')
    const router = useRouter()
    return (
        <nav>
            <div className='container m-auto'>
                <div className='px-8 py-4 flex'>
                    <div className='flex flex-col justify-center'>
                        <p className='text-4xl'>Dejavu</p>
                    </div>
                    <div className='flex-1 flex justify-center'>
                        <div>
                            <TextField
                                className='w-96'
                                placeholder="Search Keywords"
                                onChange={(e) => setSearchText(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter') {
                                        router.push(`/search?text=${searchText}`)
                                    }
                                }}
                                InputProps={{
                                    endAdornment: <InputAdornment position="end">
                                        <IconButton
                                            className='m-2'
                                            onClick={() => {
                                                router.push(`/search?text=${searchText}`)
                                            }}
                                            edge="end"
                                        >
                                            <SearchIcon />
                                        </IconButton>
                                    </InputAdornment>
                                }}
                            />
                        </div>
                    </div>
                    <div className='px-12'>
                        <div className='flex flex-col justify-center h-full'>
                            <a className='text-xl' href="https://github.com/strrl/dejavu" target='_blank'>🌟 Star on GitHub</a>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    );
}
